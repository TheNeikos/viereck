use anyhow::Context as AnyhowContext;
use piet::RenderContext;
use std::future::Future;
use std::os::unix::io::RawFd;
use std::rc::Rc;
use xcb_util::ewmh;
use mio::unix::EventedFd;
use mio::{PollOpt, Ready, Token};
use tokio::stream::Stream;
use std::task::{Poll, Context};
use std::pin::Pin;
// The following two functions are from https://github.com/mjkillough/cnx/blob/master/src/bar.rs

use crate::object::Object;

fn get_root_visual_type(conn: &xcb::Connection, screen: &xcb::Screen<'_>) -> xcb::Visualtype {
    for root in conn.get_setup().roots() {
        for allowed_depth in root.allowed_depths() {
            for visual in allowed_depth.visuals() {
                if visual.visual_id() == screen.root_visual() {
                    return visual;
                }
            }
        }
    }
    panic!("No visual type found");
}

/// Creates a `cairo::Surface` for the XCB window with the given `id`.
fn cairo_surface_for_xcb_window(
    conn: &xcb::Connection,
    screen: &xcb::Screen<'_>,
    id: u32,
    width: i32,
    height: i32,
) -> cairo::XCBSurface {
    let cairo_conn = unsafe {
        cairo::XCBConnection::from_raw_none(conn.get_raw_conn() as *mut cairo_sys::xcb_connection_t)
    };
    let visual = unsafe {
        cairo::XCBVisualType::from_raw_none(
            &mut get_root_visual_type(conn, screen).base as *mut xcb::ffi::xcb_visualtype_t
                as *mut cairo_sys::xcb_visualtype_t,
        )
    };
    let drawable = cairo::XCBDrawable(id);
    cairo::XCBSurface::create(&cairo_conn, &drawable, &visual, width, height)
}

pub struct Window {
    ewmh_connection: Rc<ewmh::Connection>,
    window: u32,
    context: cairo::Context,
}

impl Window {
    pub fn new(x: i16, y: i16, width: u16, height: u16) -> anyhow::Result<Window> {
        let (connection, _) = xcb::Connection::connect(None)?;

        let setup = connection.get_setup();
        let mut roots = setup.roots();
        let screen = roots.next().context("Couldn't find screen")?;

        let window = connection.generate_id();

        xcb::create_window(
            &connection,
            xcb::COPY_FROM_PARENT as u8,
            window,
            screen.root(),
            x,      // x
            y,      // y
            width,  // width
            height, // height
            0,      // Border
            xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
            screen.root_visual(),
            &[(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_EXPOSURE)],
        );

        let surface =
            cairo_surface_for_xcb_window(&connection, &screen, window, width.into(), height.into());

        let ewmh_connection = ewmh::Connection::connect(connection)
            .map_err(|(e, _)| e)
            .context("Could not get ewmh::Connection")?;

        ewmh::set_wm_window_type(
            &ewmh_connection,
            window,
            &[ewmh_connection.WM_WINDOW_TYPE_DOCK()],
        );

        let context = cairo::Context::new(&surface);

        xcb::map_window(&ewmh_connection, window).request_check()?;

        Ok(Window {
            ewmh_connection: Rc::new(ewmh_connection),
            window,
            context,
        })
    }

    pub fn draw(&mut self, objects: &Object) -> anyhow::Result<()> {
        let mut crc = piet_cairo::CairoRenderContext::new(&mut self.context);
        crc.clear(piet::Color::WHITE);

        let brush = crc.solid_brush(piet::Color::rgb8(0, 0, 0x80));
        crc.stroke(
            piet::kurbo::Line::new((0.0, 0.0), (150.0, 150.0)),
            &brush,
            1.0,
        );

        xcb::map_window(&self.ewmh_connection, self.window).request_check()?;
        self.ewmh_connection.flush();

        Ok(())
    }

    pub fn event_stream(&self) -> anyhow::Result<WindowEventStream> {
        Ok(WindowEventStream {
            poll_evented: tokio::io::PollEvented::new(XcbEvented(self.ewmh_connection.clone()))?,
        })
    }
}

pub enum WindowEvent {
    Draw,
    Quit,
    Unknown,
}

struct XcbEvented(Rc<ewmh::Connection>);

impl mio::event::Evented for XcbEvented {
    fn register(
        &self,
        poll: &mio::Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> std::io::Result<()> {
        EventedFd(&self.fd()).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &mio::Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> std::io::Result<()> {
        EventedFd(&self.fd()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &mio::Poll) -> std::io::Result<()> {
        EventedFd(&self.fd()).deregister(poll)
    }
}

impl XcbEvented {
    fn fd(&self) -> RawFd {
        let conn: &xcb::Connection = &self.0;
        unsafe { xcb::ffi::xcb_get_file_descriptor(conn.get_raw_conn()) }
    }
}

pub struct WindowEventStream {
    poll_evented: tokio::io::PollEvented<XcbEvented>,
}

impl Stream for WindowEventStream {
    type Item = anyhow::Result<WindowEvent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let ready = Ready::readable();

        match self.poll_evented.poll_read_ready(cx, ready) {
            Poll::Pending => return Poll::Pending,
            _ => (),
        }

        match self.poll_evented.get_ref().0.poll_for_event() {
            Some(ev) => {
                if ev.response_type() & !0x80 == xcb::EXPOSE {
                    return Poll::Ready(Some(Ok(WindowEvent::Draw)));
                } else {
                    return Poll::Ready(Some(Ok(WindowEvent::Unknown)));
                }
            }
            None => {
                self.poll_evented.clear_read_ready(cx, ready);
                return Poll::Pending;
            }
        }
    }
}

#![allow(unsafe_code)]
use anyhow::Context as AnyhowContext;
use futures::stream::Stream;
use mio::unix::EventedFd;
use mio::{PollOpt, Ready, Token};
use piet::RenderContext;
use std::os::unix::io::RawFd;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use stretch::Stretch;
use xcb_util::ewmh;
// The following two functions are from https://github.com/mjkillough/cnx/blob/master/src/bar.rs

use crate::draw;
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
    let root_visual: *mut xcb::ffi::xcb_visualtype_t = &mut get_root_visual_type(conn, screen).base;
    let visual = unsafe {
        cairo::XCBVisualType::from_raw_none(
                root_visual as *mut cairo_sys::xcb_visualtype_t,
        )
    };
    let drawable = cairo::XCBDrawable(id);
    cairo::XCBSurface::create(&cairo_conn, &drawable, &visual, width, height)
}

#[derive(Debug)]
struct NodeObject {
    node: stretch::node::Node,
    object: Option<Object>,
    children: Vec<NodeObject>,
}

impl NodeObject {
    fn new(node: stretch::node::Node, object: Option<Object>) -> NodeObject {
        NodeObject {
            node,
            object,
            children: vec![],
        }
    }

    fn add_child(&mut self, obj: NodeObject) {
        self.children.push(obj);
    }
}

pub struct Window {
    ewmh_connection: Rc<ewmh::Connection>,
    window: u32,
    context: cairo::Context,
    width: u16,
    height: u16,
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
            width,
            height,
        })
    }

    pub fn draw(&mut self, root_objects: Vec<Object>) -> anyhow::Result<()> {
        let mut crc = piet_cairo::CairoRenderContext::new(&mut self.context);
        crc.clear(piet::Color::WHITE);

        let mut stretch = Stretch::new();

        let root_node = stretch
            .new_node(draw::root_style(), vec![])
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        let mut root_obj = NodeObject::new(root_node, None);

        fn create_node_objects(
            stretch: &mut Stretch,
            root: &mut NodeObject,
            children: Vec<Object>,
        ) -> anyhow::Result<()> {
            for child in children {
                let node = {
                    match child {
                        Object::Container { .. } => stretch
                            .new_node(child.get_style(), vec![])
                            .map_err(|e| anyhow::anyhow!(e.to_string()))?,
                        Object::Text { .. } => {
                            let child = child.clone();
                            stretch
                                .new_leaf(
                                    child.get_style(),
                                    Box::new(move |size| child.compute_size(size)),
                                )
                                .map_err(|e| anyhow::anyhow!(e.to_string()))?
                        }
                    }
                };

                let mut nobj = NodeObject::new(node, Some(child));

                if let Some(ref mut obj) = &mut nobj.object {
                    match obj {
                        Object::Container {
                            ref mut children, ..
                        } => {
                            let children = std::mem::replace(children, vec![]);
                            create_node_objects(stretch, &mut nobj, children)?;
                        }
                        _ => (),
                    }
                }
                stretch
                    .add_child(root.node, node)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
                root.add_child(nobj);
            }

            Ok(())
        }

        create_node_objects(&mut stretch, &mut root_obj, root_objects)?;

        stretch
            .compute_layout(
                root_node,
                stretch::geometry::Size {
                    width: stretch::number::Number::Defined(self.width as f32),
                    height: stretch::number::Number::Defined(self.height as f32),
                },
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        let root_layout = stretch
            .layout(root_node)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        draw::draw_rectangle(
            &mut crc,
            root_layout.location.x,
            root_layout.location.y,
            root_layout.size.width,
            root_layout.size.height,
            &piet::Color::grey8(0xDD),
        );

        fn draw_node_objects(
            stretch: &Stretch,
            rc: &mut impl RenderContext,
            obj: NodeObject,
        ) -> anyhow::Result<()> {
            let node_layout = stretch
                .layout(obj.node)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;

            if let Some(obj) = obj.object {
                if let Some(color) = &obj.get_background() {
                    draw::draw_rectangle(
                        rc,
                        node_layout.location.x,
                        node_layout.location.y,
                        node_layout.size.width,
                        node_layout.size.height,
                        color,
                    );
                }

                match obj {
                    Object::Container { .. } => {
                        // Do nothing as its just a container
                    }
                    Object::Text {
                        text,
                        font,
                        font_size,
                        color,
                        ..
                    } => {
                        use piet::{FontBuilder, Text, TextLayoutBuilder};

                        let text_builder = rc.text();
                        let font = text_builder
                            .new_font_by_name(&font, font_size)
                            .build()
                            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
                        let text_layout = text_builder
                            .new_text_layout(&font, &text)
                            .build()
                            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
                        let brush = rc.solid_brush(color);
                        // draw::draw_rectangle(
                        //     rc,
                        //     node_layout.location.x,
                        //     node_layout.location.y,
                        //     node_layout.size.width,
                        //     node_layout.size.height,
                        //     &piet::Color::rgba(0xDD, 0xDD, 0xDD, 100),
                        // );
                        let half_height = node_layout.size.height;
                        rc.draw_text(
                            &text_layout,
                            (
                                node_layout.location.x as f64,
                                (node_layout.location.y + half_height) as f64,
                            ),
                            &brush,
                        );
                    }
                }
            }
            rc.save().map_err(|e| anyhow::anyhow!(e.to_string()))?;
            rc.transform(kurbo::Affine::translate((
                node_layout.location.x as f64,
                node_layout.location.y as f64,
            )));
            rc.clip(kurbo::Rect::new(
                0.,
                0.,
                node_layout.size.width as f64,
                node_layout.size.height as f64,
            ));
            for child in obj.children {
                draw_node_objects(stretch, rc, child)?;
            }
            rc.restore().map_err(|e| anyhow::anyhow!(e.to_string()))?;
            Ok(())
        }

        draw_node_objects(&stretch, &mut crc, root_obj)?;

        crc.finish().map_err(|e| anyhow::anyhow!(e.to_string()))?;

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
                self.poll_evented.clear_read_ready(cx, ready)?;
                return Poll::Pending;
            }
        }
    }
}

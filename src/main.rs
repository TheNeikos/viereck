use structopt::StructOpt;
use tokio::stream::StreamExt;

mod object;
mod window;

#[derive(Debug, StructOpt)]
#[structopt(name = "vierecl", about = "Viereck is a versatile drawing program")]
struct CmdOptions {
    /// X position
    #[structopt(short, long, default_value = "0")]
    x: i16,
    /// Y position
    #[structopt(short, long, default_value = "0")]
    y: i16,
    /// Width position
    #[structopt(short, long)]
    width: u16,
    /// Height position
    #[structopt(short, long)]
    height: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = CmdOptions::from_args();

    let mut win = window::Window::new(opt.x, opt.y, opt.width, opt.height)?;

    let objs = object::Object;

    let mut events = win.event_stream()?;

    while let Some(Ok(ev)) = events.next().await {
        match ev {
            window::WindowEvent::Draw => {
                win.draw(&objs)?;
            }
            window::WindowEvent::Quit => {
                break;
            }
            _ => {
                // Unknown event? Not cared
                println!("Unknown event");
            }
        }
    }

    Ok(())
}

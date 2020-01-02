use futures::stream::StreamExt as FStreamExt;
use structopt::StructOpt;
use tokio::io::AsyncBufReadExt;

mod draw;
pub mod object;
mod window;

#[derive(Debug, StructOpt)]
#[structopt(name = "viereck", about = "Viereck is a versatile drawing program")]
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

    let mut root_objs = vec![
    ];

    enum Events {
        Window(window::WindowEvent),
        Input(Vec<object::Object>),
    }

    let windows_events = win.event_stream()?.map(|e| e.map(Events::Window));

    let input_events = tokio::io::BufReader::new(tokio::io::stdin())
        .lines()
        .map(|l| match l {
            Ok(line) => Ok(serde_json::from_str(&line).map(Events::Input)?),
            Err(e) => Err(e)?,
        });

    let mut events = futures::stream::select(windows_events, input_events);

    while let Some(ev) = events.next().await {
        match ev {
            Ok(Events::Window(ev)) => {
                match ev {
                    window::WindowEvent::Draw => {
                        win.draw(root_objs.clone())?;
                    }
                    _ => {
                        // Unknown event? Not cared
                        eprintln!("Unknown event");
                    }
                }
            }
            Ok(Events::Input(new_objs)) => {
                root_objs = new_objs;
                win.draw(root_objs.clone())?;
            }
            Err(e) => {
                match e.downcast_ref() {
                    Some(serde_json::error::Error { .. }) => {
                        eprintln!("Could not parse input: {}", e);
                        e.chain().skip(1).for_each(|c| eprintln!("because: {}", c));
                        continue;
                    }
                    _ => (),
                }

                eprintln!("Unknown error: {}", e);
                e.chain().skip(1).for_each(|c| eprintln!("because: {}", c));
            }
        }
    }

    Ok(())
}

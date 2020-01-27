use anyhow::Context;
use futures::stream::StreamExt as FStreamExt;
use futures::Stream;
use time::PrimitiveDateTime;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;

use viereck::Object as VObject;

type Object = VObject<viereck::style::Style>;

fn get_command_stream(
    command: &'static str,
) -> anyhow::Result<impl Stream<Item = std::io::Result<String>>> {
    let mut child = Command::new(command)
        .arg("--idle")
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .context("Could not get stdout from child")?;

    tokio::spawn(async {
        let _ = child.await.unwrap();
    });

    Ok(tokio::io::BufReader::new(stdout).lines())
}

async fn get_command_output(
    command: &'static str,
    args: &[&'static str],
) -> anyhow::Result<String> {
    let child = Command::new(command).args(args).output().await?;
    Ok(std::string::String::from_utf8(child.stdout).context("Command output was not utf-8")?)
}

fn get_herbstclient_information() -> anyhow::Result<impl Stream<Item = std::io::Result<Vec<String>>>>
{
    Ok(get_command_stream("herbstclient")?
        .map(|tags| Ok(tags?.split('\t').map(|t| t.to_owned()).collect())))
}

fn get_current_time() -> impl Stream<Item = PrimitiveDateTime> {
    tokio::time::interval(tokio::time::Duration::from_secs(1)).map(|_| PrimitiveDateTime::now())
}

async fn filter_map_result<T, E>(res: Result<T, E>) -> Option<T> {
    res.ok()
}

fn redraw_panel(panel: &Panel) -> anyhow::Result<Vec<Object>> {
    let mut objs: Vec<Object> = vec![];

    objs.push({
        Object::Container {
            background: Some(piet::Color::BLACK),
            style: Default::default(),
            children: panel
                .tags
                .iter()
                .map(|tag| {
                    let (fg, bg) = color_for_tag(tag.0);

                    Object::Container {
                        background: Some(bg),
                        style: viereck::style::Style {
                            padding: Some(stretch::geometry::Rect {
                                start: stretch::style::Dimension::Points(5.),
                                end: stretch::style::Dimension::Points(5.),
                                top: stretch::style::Dimension::Points(3.),
                                bottom: stretch::style::Dimension::Points(3.),
                            }),
                            ..Default::default()
                        },
                        children: vec![Object::Text {
                            font: "Noto Sans Mono".into(),
                            text: tag.1.clone(),
                            font_size: 12.,
                            color: fg,
                            style: Default::default(),
                        }],
                        corner_radius: None,
                    }
                })
                .collect(),
            corner_radius: None,
        }
    });

    objs.push({
        Object::Container {
            background: Some(piet::Color::BLACK),
            style: viereck::style::Style {
                flex_grow: Some(1.),
                padding: Some(stretch::geometry::Rect {
                    start: stretch::style::Dimension::Points(5.),
                    end: stretch::style::Dimension::Points(5.),
                    top: stretch::style::Dimension::Points(3.),
                    bottom: stretch::style::Dimension::Points(3.),
                }),
                ..Default::default()
            },
            children: vec![Object::Text {
                font: "Noto Sans Mono".into(),
                text: panel.title.clone(),
                font_size: 12.,
                color: piet::Color::WHITE,
                style: viereck::style::Style {
                    align_content: Some(stretch::style::AlignContent::Center),
                    ..Default::default()
                },
            }],
            corner_radius: None,
        }
    });

    objs.push({
        Object::Container {
            background: Some(piet::Color::BLACK),
            style: viereck::style::Style {
                padding: Some(stretch::geometry::Rect {
                    start: stretch::style::Dimension::Points(1.),
                    end: stretch::style::Dimension::Points(1.),
                    top: stretch::style::Dimension::Points(1.),
                    bottom: stretch::style::Dimension::Points(1.),
                }),
                ..Default::default()
            },
            children: {
                let mut childs = vec![];

                for battery in battery::Manager::new()?.batteries()? {
                    let pct = battery?
                        .state_of_charge()
                        .get::<battery::units::ratio::percent>()
                        as u8;

                    let bg = match pct {
                        21..=50 => piet::Color::rgb8(0xff, 0xb8, 0x61),
                        0..=20 => piet::Color::rgb8(0xff, 0x69, 0x61),
                        _ => piet::Color::WHITE,
                    };

                    let pct = pct as f32 / 100.;

                    childs.push(Object::Container {
                        background: Some(piet::Color::grey8(0x55)),
                        style: viereck::style::Style {
                            align_self: Some(stretch::style::AlignSelf::Stretch),
                            padding: Some(stretch::geometry::Rect {
                                start: stretch::style::Dimension::Points(1.),
                                end: stretch::style::Dimension::Points(1.),
                                top: stretch::style::Dimension::Points(1.),
                                bottom: stretch::style::Dimension::Points(1.),
                            }),
                            ..Default::default()
                        },
                        children: vec![Object::Container {
                            background: Some(bg),
                            style: viereck::style::Style {
                                align_self: Some(stretch::style::AlignSelf::FlexEnd),
                                size: Some(stretch::geometry::Size {
                                    width: stretch::style::Dimension::Points(5.),
                                    height: stretch::style::Dimension::Percent(pct),
                                }),
                                ..Default::default()
                            },
                            children: vec![],
                            corner_radius: Some(1.),
                        }],
                        corner_radius: None,
                    })
                }

                childs.push(Object::Text {
                    font: "Noto Sans Mono".into(),
                    text: panel.time.format("%F %T"),
                    font_size: 12.,
                    color: piet::Color::WHITE,
                    style: viereck::style::Style {
                        margin: Some(stretch::geometry::Rect {
                            start: stretch::style::Dimension::Points(5.),
                            end: stretch::style::Dimension::Points(5.),
                            top: stretch::style::Dimension::Points(3.),
                            bottom: stretch::style::Dimension::Points(3.),
                        }),
                        justify_content: Some(stretch::style::JustifyContent::Center),
                        ..Default::default()
                    },
                });

                childs
            },
            corner_radius: None,
        }
    });

    Ok(objs)
}

async fn get_herbstclient_tags() -> anyhow::Result<Vec<(Tag, String)>> {
    Ok(get_command_output("herbstclient", &["tag_status"])
        .await?
        .trim()
        .split('\t')
        .map(|t| {
            (
                match t.get(0..1) {
                    Some(".") => Tag::Empty,
                    Some(":") => Tag::NotEmpty,
                    Some("+") => Tag::NotFocused,
                    Some("#") => Tag::Focused,
                    Some("-") => Tag::NotFocusedThere,
                    Some("%") => Tag::FocusedThere,
                    Some("!") => Tag::Alterting,
                    Some(_) | None => Tag::Empty,
                },
                t.get(1..).map(ToOwned::to_owned).unwrap_or_default(),
            )
        })
        .collect())
}

fn color_for_tag(tag: Tag) -> (piet::Color, piet::Color) {
    match tag {
        Tag::Focused => (
            piet::Color::rgb8(0x10, 0x10, 0x10),
            piet::Color::rgb8(0x9F, 0xBC, 0x00),
        ),
        Tag::NotFocused => (
            piet::Color::rgb8(0xFF, 0xFF, 0xFF),
            piet::Color::rgb8(0x00, 0x00, 0x00),
        ),
        Tag::NotEmpty => (
            piet::Color::rgb8(0xFF, 0xFF, 0xFF),
            piet::Color::rgb8(0x77, 0x77, 0x77),
        ),
        Tag::Alterting => (
            piet::Color::rgb8(0xFF, 0xFF, 0xFF),
            piet::Color::rgb8(0xFF, 0x00, 0x00),
        ),
        _ => (
            piet::Color::rgb8(0xDD, 0xDD, 0xDD),
            piet::Color::rgb8(0x00, 0x00, 0x00),
        ),
    }
}

async fn update_panel(panel: &mut Panel, ev: Event) -> anyhow::Result<()> {
    match ev {
        Event::Tags(ts) => match ts[0].as_ref() {
            "tag_changed" | "tag_flags" => panel.tags = get_herbstclient_tags().await?,
            "window_title_changed" | "focus_changed" => panel.title = ts[2].clone(),
            _ => {}
        },
        Event::Time(t) => panel.time = t,
    }

    Ok(())
}

#[derive(Debug, Copy, Clone)]
enum Tag {
    Empty,
    NotEmpty,
    NotFocused,
    Focused,
    NotFocusedThere,
    FocusedThere,
    Alterting,
}

#[derive(Debug)]
struct Panel {
    tags: Vec<(Tag, String)>,
    title: String,
    time: PrimitiveDateTime,
}

enum Event {
    Tags(Vec<String>),
    Time(PrimitiveDateTime),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let hc_tags = get_herbstclient_information()?
        .filter_map(filter_map_result)
        .map(Event::Tags);
    let current_date = get_current_time().map(Event::Time);

    let mut events = Box::pin(futures::stream::select(hc_tags, current_date));

    let mut panel = Panel {
        tags: get_herbstclient_tags().await?,
        title: String::new(),
        time: PrimitiveDateTime::now(),
    };

    while let Some(ev) = events.next().await {
        update_panel(&mut panel, ev).await?;
        let p = redraw_panel(&panel)?;
        println!("{}", serde_json::to_string(&p)?);
    }

    Ok(())
}

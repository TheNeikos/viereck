use anyhow::{Result, anyhow};
use structopt::StructOpt;
use serde_json::{to_string, from_str};

use viereck::object::Object;

pub fn parse_dimension(input: &str) -> Result<stretch::style::Dimension> {
    if let "auto" = input {
        return Ok(stretch::style::Dimension::Auto);
    }

    if let Ok(pts) = input.parse() {
        return Ok(stretch::style::Dimension::Points(pts));
    }

    if input.ends_with('%') {
        if let Ok(pct) = input.split('%').next().unwrap().parse() {
            return Ok(stretch::style::Dimension::Percent(pct));
        }
    }

    Err(anyhow!("Could not parse: {}", input))
}

pub fn parse_object(input: &str) -> Result<Object> {
    Ok(from_str(input)?)
}

pub fn parse_hex(input: &str) -> Result<u32> {
    Ok(u32::from_str_radix(input.trim_start_matches("0x"), 16)?)
}

#[derive(Debug, StructOpt)]
#[structopt(name = "viereck-container", about = "viereck-container makes it easy to create viereck containers")]
struct CmdOptions {
    /// Width position
    #[structopt(short, long, parse(try_from_str = parse_dimension))]
    width: Option<stretch::style::Dimension>,
    /// Height position
    #[structopt(short, long, parse(try_from_str = parse_dimension))]
    height: Option<stretch::style::Dimension>,
    /// Grow the container if there is space
    #[structopt(short, long)]
    grow: Option<f32>,
    /// Shrink the container if there is not enough space
    #[structopt(short, long)]
    shrink: Option<f32>,
    /// Font
    #[structopt(short, long)]
    font: String,
    /// Font Size
    #[structopt(short = "-z", long)]
    font_size: f64,
    /// Text
    #[structopt(short, long)]
    text: String,
    /// Text color
    #[structopt(short, long, parse(try_from_str = parse_hex))]
    color: u32,
}

fn main() -> Result<()> {
    let opt = CmdOptions::from_args();

    let obj = Object::Text {
        style: Some(stretch::style::Style {
            size: stretch::geometry::Size {
                width: opt.width.unwrap_or(stretch::style::Dimension::Auto),
                height: opt.height.unwrap_or(stretch::style::Dimension::Auto),
            },
            flex_grow: opt.grow.unwrap_or_default(),
            flex_shrink: opt.shrink.unwrap_or_default(),
            align_self: stretch::style::AlignSelf::Baseline,
            ..Default::default()
        }),
        text: opt.text,
        font: opt.font,
        color: piet::Color::from_rgba32_u32(opt.color),
        font_size: opt.font_size,
    };

    println!("{}", to_string(&obj)?);

    Ok(())
}


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
    #[structopt(long, parse(try_from_str = parse_dimension))]
    width: Option<stretch::style::Dimension>,
    /// Height position
    #[structopt(long, parse(try_from_str = parse_dimension))]
    height: Option<stretch::style::Dimension>,
    /// Grow the container if there is space
    #[structopt(long)]
    grow: Option<f32>,
    /// Shrink the container if there is not enough space
    #[structopt(long)]
    shrink: Option<f32>,
    /// Margin (to the outside container)
    #[structopt(long, parse(try_from_str = parse_dimension))]
    margin: Option<stretch::style::Dimension>,
    /// Padding (to the inside content)
    #[structopt(long, parse(try_from_str = parse_dimension))]
    padding: Option<stretch::style::Dimension>,
    /// Child contents
    #[structopt(short, long, parse(try_from_str = parse_object))]
    children: Vec<Object>,
    /// Background 
    /// 
    /// In rgba hex format 0xXXXXXXXX
    #[structopt(short, long, parse(try_from_str = parse_hex))]
    background: Option<u32>,
}

fn main() -> Result<()> {
    let opt = CmdOptions::from_args();

    let obj = Object::Container {
        style: stretch::style::Style {
            size: stretch::geometry::Size {
                width: opt.width.unwrap_or(stretch::style::Dimension::Auto),
                height: opt.height.unwrap_or(stretch::style::Dimension::Auto),
            },
            margin: stretch::geometry::Rect {
                start: opt.margin.unwrap_or_default(),
                end: opt.margin.unwrap_or_default(),
                top: opt.margin.unwrap_or_default(),
                bottom: opt.margin.unwrap_or_default(),
            },
            padding: stretch::geometry::Rect {
                start: opt.padding.unwrap_or_default(),
                end: opt.padding.unwrap_or_default(),
                top: opt.padding.unwrap_or_default(),
                bottom: opt.padding.unwrap_or_default(),
            },
            flex_grow: opt.grow.unwrap_or_default(),
            flex_shrink: opt.shrink.unwrap_or_default(),
            ..Default::default()
        },
        background: opt.background.map(piet::Color::from_rgba32_u32),
        children: opt.children,
    };

    println!("{}", to_string(&obj)?);

    Ok(())
}

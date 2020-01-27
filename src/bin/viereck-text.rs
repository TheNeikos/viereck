use anyhow::{anyhow, Result};
use serde_json::{from_str, to_string};
use structopt::StructOpt;

mod common;

use common::style::Style;

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
#[structopt(
    name = "viereck-container",
    about = "viereck-container makes it easy to create viereck containers"
)]
struct CmdOptions {
    #[structopt(flatten)]
    style: common::style::StyleOpts,
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

    let obj = Object::<Style>::Text {
        style: opt.style.to_style(),
        text: opt.text,
        font: opt.font,
        color: piet::Color::from_rgba32_u32(opt.color),
        font_size: opt.font_size,
    };

    println!("{}", to_string(&obj)?);

    Ok(())
}

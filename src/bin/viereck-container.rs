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
    /// Child contents
    #[structopt(short, long, parse(try_from_str = parse_object))]
    children: Vec<Object>,
    /// Background 
    /// 
    /// In rgba hex format 0xXXXXXXXX
    #[structopt(short, long, parse(try_from_str = parse_hex))]
    background: Option<u32>
}

fn main() -> Result<()> {
    let opt = CmdOptions::from_args();

    eprintln!("{:#?}", opt);

    let obj = Object::Container {
        style: stretch::style::Style {
            size: stretch::geometry::Size {
                width: opt.width.unwrap_or(stretch::style::Dimension::Auto),
                height: opt.height.unwrap_or(stretch::style::Dimension::Auto),
            },
            ..Default::default()
        },
        background: opt.background.map(piet::Color::from_rgba32_u32),
        children: opt.children,
    };

    println!("{}", to_string(&obj)?);

    Ok(())
}

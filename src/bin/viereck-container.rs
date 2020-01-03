use anyhow::Result;
use structopt::StructOpt;
use serde_json::{to_string, from_str};

mod common;

use viereck::object::Object;

fn parse_object(input: &str) -> Result<Object> {
    Ok(from_str(input)?)
}

fn parse_hex(input: &str) -> Result<u32> {
    Ok(u32::from_str_radix(input.trim_start_matches("0x"), 16)?)
}

#[derive(Debug, StructOpt)]
#[structopt(name = "viereck-container", about = "viereck-container makes it easy to create viereck containers")]
struct CmdOptions {
    #[structopt(flatten)]
    style: common::style::StyleOpts,
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
        style: opt.style.to_style(),
        background: opt.background.map(piet::Color::from_rgba32_u32),
        children: opt.children,
    };

    println!("{}", to_string(&obj)?);

    Ok(())
}

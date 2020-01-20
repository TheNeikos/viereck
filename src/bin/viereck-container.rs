use anyhow::Result;
use serde_derive::Serialize;
use serde_json::to_string;
use structopt::StructOpt;

mod common;
use common::style::Style;

fn parse_hex(input: &str) -> Result<u32> {
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
    /// Child contents
    #[structopt(short, long)]
    children: Vec<serde_json::Value>,
    /// Background
    ///
    /// In rgba hex format 0xXXXXXXXX
    #[structopt(short, long, parse(try_from_str = parse_hex))]
    background: Option<u32>,
}

//
// KEEP THE BELOW ENUM IN SYNC!!
//
mod opt_external_color {
    use serde::{Serialize, Serializer};
    use viereck::object::ColorDef;

    pub fn serialize<S>(value: &Option<piet::Color>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a>(#[serde(with = "ColorDef")] &'a piet::Color);

        value.as_ref().map(Helper).serialize(serializer)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
enum Object {
    Container {
        children: Vec<serde_json::Value>,
        style: Style,
        #[serde(default, with = "opt_external_color")]
        background: Option<piet::Color>,
    },
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

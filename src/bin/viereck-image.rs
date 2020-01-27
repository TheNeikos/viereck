use anyhow::Result;
use image::GenericImageView;
use serde_json::to_string;
use stretch::style::Dimension;
use structopt::StructOpt;

mod common;

use common::style::Style;

use viereck::object::Object;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "viereck-image",
    about = "viereck-image makes it easy to create viereck images"
)]
struct CmdOptions {
    #[structopt(flatten)]
    style: common::style::StyleOpts,
    /// Path to PNG
    #[structopt(short, long)]
    path: String,
    /// Do not read the size of the image
    ///
    /// If this is not set, the file is accessed and its size read
    #[structopt(short, long)]
    no_size_read: bool,
    /// Allow stretching of the image
    ///
    /// If set, width/height ratio is preserved, this only works if height and
    /// width are set!
    #[structopt(short, long)]
    allow_deform: bool,
}

fn main() -> Result<()> {
    let mut opt = CmdOptions::from_args();

    let file = std::fs::File::open(&opt.path)?;

    let img = image::load(std::io::BufReader::new(file), image::ImageFormat::PNG)?;

    let (width, height) = img.dimensions();

    if !opt.no_size_read {
        opt.style.height = Some(Dimension::Points(height as f32));
        opt.style.width = Some(Dimension::Points(width as f32));
    }

    if !opt.allow_deform {
        opt.style.aspect_ratio = match (opt.style.width, opt.style.height) {
            (
                Some(stretch::style::Dimension::Points(w)),
                Some(stretch::style::Dimension::Points(h)),
            ) => Some(stretch::number::Number::Defined(w / h)),
            _ => None,
        }
    }

    let obj = Object::<Style>::Image {
        style: opt.style.to_style(),
        path: opt.path,
    };

    println!("{}", to_string(&obj)?);

    Ok(())
}

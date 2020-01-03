use structopt::StructOpt;
use anyhow::{Result, anyhow};

fn parse_align_items(input: &str) -> Result<stretch::style::AlignItems> {
    Ok(match input {
        "flex_start" => stretch::style::AlignItems::FlexStart,
        "flex_end" => stretch::style::AlignItems::FlexEnd,
        "center" => stretch::style::AlignItems::Center,
        "baseline" => stretch::style::AlignItems::Baseline,
        "stretch" => stretch::style::AlignItems::Stretch,
        _ => return Err(anyhow!("{} needs to be one of: 'flex_start', 'flex_end', 'center', 'baseline', 'stretch'", input)),
    })
}

fn parse_dimension(input: &str) -> Result<stretch::style::Dimension> {
    if let "auto" = input {
        return Ok(stretch::style::Dimension::Auto);
    }

    if let Ok(pts) = input.parse() {
        return Ok(stretch::style::Dimension::Points(pts));
    }

    if input.ends_with('%') {
        if let Ok(pct) = input.split('%').next().unwrap().parse::<f32>() {
            return Ok(stretch::style::Dimension::Percent(pct / 100.0));
        }
    }

    Err(anyhow!("{} is not a dimension, expected 'auto', a number (representing pixels) or percentage (0% - 100%)", input))
}


#[derive(Debug, StructOpt)]
pub struct StyleOpts {
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
    /// Padding end (the right side if ltr)
    #[structopt(long, parse(try_from_str = parse_dimension))]
    padding_end: Option<stretch::style::Dimension>,
    /// Padding start (the left side if ltr)
    #[structopt(long, parse(try_from_str = parse_dimension))]
    padding_start: Option<stretch::style::Dimension>,
    /// Padding top
    #[structopt(long, parse(try_from_str = parse_dimension))]
    padding_top: Option<stretch::style::Dimension>,
    /// Padding bottom
    #[structopt(long, parse(try_from_str = parse_dimension))]
    padding_bottom: Option<stretch::style::Dimension>,
    /// How to align items inside
    #[structopt(long, parse(try_from_str = parse_align_items))]
    align_items: Option<stretch::style::AlignItems>,
}

impl StyleOpts {
    pub fn to_style(&self) -> stretch::style::Style {
        stretch::style::Style {
            size: stretch::geometry::Size {
                width: self.width.unwrap_or(stretch::style::Dimension::Auto),
                height: self.height.unwrap_or(stretch::style::Dimension::Auto),
            },
            margin: stretch::geometry::Rect {
                start: self.margin.unwrap_or_default(),
                end: self.margin.unwrap_or_default(),
                top: self.margin.unwrap_or_default(),
                bottom: self.margin.unwrap_or_default(),
            },
            padding: stretch::geometry::Rect {
                start: self.padding_start.or(self.padding).unwrap_or_default(),
                end: self.padding_end.or(self.padding).unwrap_or_default(),
                top: self.padding_top.or(self.padding).unwrap_or_default(),
                bottom: self.padding_bottom.or(self.padding).unwrap_or_default(),
            },
            flex_grow: self.grow.unwrap_or_default(),
            flex_shrink: self.shrink.unwrap_or_default(),
            align_items: self.align_items.unwrap_or_default(),
            ..Default::default()
        }
    }
}

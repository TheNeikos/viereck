use anyhow::{anyhow, Result};
use serde_derive::Serialize;
use structopt::StructOpt;

fn parse_align_items(input: &str) -> Result<stretch::style::AlignItems> {
    Ok(match input {
        "flex_start" => stretch::style::AlignItems::FlexStart,
        "flex_end" => stretch::style::AlignItems::FlexEnd,
        "center" => stretch::style::AlignItems::Center,
        "baseline" => stretch::style::AlignItems::Baseline,
        "stretch" => stretch::style::AlignItems::Stretch,
        _ => {
            return Err(anyhow!(
                "{} needs to be one of: 'flex_start', 'flex_end', 'center', 'baseline', 'stretch'",
                input
            ))
        }
    })
}

fn parse_align_self(input: &str) -> Result<stretch::style::AlignSelf> {
    Ok(match input {
        "auto" => stretch::style::AlignSelf::FlexStart,
        "flex_start" => stretch::style::AlignSelf::FlexStart,
        "flex_end" => stretch::style::AlignSelf::FlexEnd,
        "center" => stretch::style::AlignSelf::Center,
        "baseline" => stretch::style::AlignSelf::Baseline,
        "stretch" => stretch::style::AlignSelf::Stretch,
        _ => {
            return Err(anyhow!(
                "{} needs to be one of: 'auto', 'flex_start', 'flex_end', 'center', 'baseline', 'stretch'",
                input
            ))
        }
    })
}

fn parse_justify_content(input: &str) -> Result<stretch::style::JustifyContent> {
    Ok(match input {
        "flex_start" => stretch::style::JustifyContent::FlexStart,
        "flex_end" => stretch::style::JustifyContent::FlexEnd,
        "center" => stretch::style::JustifyContent::Center,
        "space_between" => stretch::style::JustifyContent::SpaceBetween,
        "space_around" => stretch::style::JustifyContent::SpaceAround,
        "space_evenly" => stretch::style::JustifyContent::SpaceEvenly,
        _ => {
            return Err(anyhow!(
                "{} needs to be one of: 'flex_start', 'flex_end', 'center', 'space_between', 'space_around', 'space_evenly'",
                input
            ))
        }
    })
}

fn parse_flex_direction(input: &str) -> Result<stretch::style::FlexDirection> {
    Ok(match input {
        "row" => stretch::style::FlexDirection::Row,
        "column" => stretch::style::FlexDirection::Column,
        "row_reverse" => stretch::style::FlexDirection::RowReverse,
        "column_reverse" => stretch::style::FlexDirection::ColumnReverse,
        _ => {
            return Err(anyhow!(
                "{} needs to be one of: 'row', 'column', 'row_reverse', 'column_reverse'",
                input
            ))
        }
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

fn parse_number(input: &str) -> Result<stretch::number::Number> {
    if let Ok(pts) = input.parse() {
        return Ok(stretch::number::Number::Defined(pts));
    }

    Err(anyhow!("{} is not a number, expected a number", input))
}

#[derive(Debug, StructOpt, Clone)]
pub struct StyleOpts {
    /// Width position
    #[structopt(long, parse(try_from_str = parse_dimension))]
    pub width: Option<stretch::style::Dimension>,
    /// Height position
    #[structopt(long, parse(try_from_str = parse_dimension))]
    pub height: Option<stretch::style::Dimension>,
    /// Grow the container if there is space
    #[structopt(long)]
    pub grow: Option<f32>,
    /// Shrink the container if there is not enough space
    #[structopt(long)]
    pub shrink: Option<f32>,
    /// Margin (to the outside container)
    #[structopt(long, parse(try_from_str = parse_dimension))]
    pub margin: Option<stretch::style::Dimension>,
    /// Padding (to the inside content)
    #[structopt(long, parse(try_from_str = parse_dimension))]
    pub padding: Option<stretch::style::Dimension>,
    /// Padding end (the right side if ltr)
    #[structopt(long, parse(try_from_str = parse_dimension))]
    pub padding_end: Option<stretch::style::Dimension>,
    /// Padding start (the left side if ltr)
    #[structopt(long, parse(try_from_str = parse_dimension))]
    pub padding_start: Option<stretch::style::Dimension>,
    /// Padding top
    #[structopt(long, parse(try_from_str = parse_dimension))]
    pub padding_top: Option<stretch::style::Dimension>,
    /// Padding bottom
    #[structopt(long, parse(try_from_str = parse_dimension))]
    pub padding_bottom: Option<stretch::style::Dimension>,
    /// How to align items inside in the cross-axis
    #[structopt(long, parse(try_from_str = parse_align_items))]
    pub align_items: Option<stretch::style::AlignItems>,
    /// How to align itself inside in the parents cross-axis
    #[structopt(long, parse(try_from_str = parse_align_self))]
    pub align_self: Option<stretch::style::AlignSelf>,
    /// How to align items inside in the main-axis
    #[structopt(long, parse(try_from_str = parse_justify_content))]
    pub justify_content: Option<stretch::style::JustifyContent>,
    /// In which direction to layout items
    #[structopt(long, parse(try_from_str = parse_flex_direction))]
    pub flex_direction: Option<stretch::style::FlexDirection>,
    /// Aspect Ratio
    #[structopt(long, parse(try_from_str = parse_number))]
    pub aspect_ratio: Option<stretch::number::Number>,
}

impl StyleOpts {
    pub fn to_style(&self) -> Style {
        Style {
            size: {
                if self.width.or(self.height).is_some() {
                    Some(stretch::geometry::Size {
                        width: self.width.unwrap_or(stretch::style::Dimension::Auto),
                        height: self.height.unwrap_or(stretch::style::Dimension::Auto),
                    })
                } else {
                    None
                }
            },
            margin: {
                if self.margin.is_some() {
                    Some(stretch::geometry::Rect {
                        start: self.margin.unwrap_or_default(),
                        end: self.margin.unwrap_or_default(),
                        top: self.margin.unwrap_or_default(),
                        bottom: self.margin.unwrap_or_default(),
                    })
                } else {
                    None
                }
            },
            padding: {
                if self
                    .padding_start
                    .or(self.padding_end)
                    .or(self.padding_top)
                    .or(self.padding_bottom)
                    .or(self.padding)
                    .is_some()
                {
                    Some(stretch::geometry::Rect {
                        start: self.padding_start.or(self.padding).unwrap_or_default(),
                        end: self.padding_end.or(self.padding).unwrap_or_default(),
                        top: self.padding_top.or(self.padding).unwrap_or_default(),
                        bottom: self.padding_bottom.or(self.padding).unwrap_or_default(),
                    })
                } else {
                    None
                }
            },
            flex_grow: self.grow,
            flex_shrink: self.shrink,
            align_items: self.align_items,
            align_self: self.align_self,
            justify_content: self.justify_content,
            flex_direction: self.flex_direction,
            aspect_ratio: self.aspect_ratio,
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Style {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<stretch::style::Display>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_type: Option<stretch::style::PositionType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<stretch::style::Direction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_direction: Option<stretch::style::FlexDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_wrap: Option<stretch::style::FlexWrap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overflow: Option<stretch::style::Overflow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_items: Option<stretch::style::AlignItems>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_self: Option<stretch::style::AlignSelf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_content: Option<stretch::style::AlignContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify_content: Option<stretch::style::JustifyContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<stretch::geometry::Rect<stretch::style::Dimension>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<stretch::geometry::Rect<stretch::style::Dimension>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<stretch::geometry::Rect<stretch::style::Dimension>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border: Option<stretch::geometry::Rect<stretch::style::Dimension>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_grow: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_shrink: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_basis: Option<stretch::style::Dimension>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<stretch::geometry::Size<stretch::style::Dimension>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_size: Option<stretch::geometry::Size<stretch::style::Dimension>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_size: Option<stretch::geometry::Size<stretch::style::Dimension>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<stretch::number::Number>,
}

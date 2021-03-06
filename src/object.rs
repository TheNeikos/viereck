use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(remote = "piet::Color")]
pub enum ColorDef {
    Rgba32(u32),
}

mod opt_external_color {
    use super::ColorDef;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<piet::Color>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a>(#[serde(with = "ColorDef")] &'a piet::Color);

        value.as_ref().map(Helper).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<piet::Color>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper(#[serde(with = "ColorDef")] piet::Color);

        let helper = Option::deserialize(deserializer)?;
        Ok(helper.map(|Helper(external)| external))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Object<S = stretch::style::Style> {
    Container {
        children: Vec<Object<S>>,
        style: S,
        #[serde(default, with = "opt_external_color")]
        background: Option<piet::Color>,
        corner_radius: Option<f64>,
    },
    Text {
        font: String,
        text: String,
        font_size: f64,
        #[serde(with = "ColorDef")]
        color: piet::Color,
        style: S,
    },
    Image {
        style: S,
        path: String,
    },
}

impl Object {
    pub fn get_style(&self) -> stretch::style::Style {
        match self {
            Self::Container { style, .. } => *style,
            Self::Text { style, .. } => *style,
            Self::Image { style, .. } => *style,
        }
    }

    pub fn get_background(&self) -> Option<&piet::Color> {
        match self {
            Self::Container { background, .. } => background.as_ref(),
            Self::Text { .. } => None,
            Self::Image { .. } => None,
        }
    }

    pub fn compute_size(
        &self,
        size: stretch::geometry::Size<stretch::number::Number>,
    ) -> Result<stretch::geometry::Size<f32>, Box<dyn std::any::Any>> {
        use piet::{FontBuilder, Text, TextLayout, TextLayoutBuilder};
        use stretch::number::MinMax;
        match self {
            Self::Text {
                font,
                text,
                font_size,
                ..
            } => {
                let mut text_builder = piet_cairo::CairoText::new();
                let font = text_builder
                    .new_font_by_name(&font, *font_size)
                    .build()
                    .unwrap();
                let text_layout = text_builder.new_text_layout(&font, &text).build().unwrap();
                let width = text_layout.width() as f32;
                Ok(stretch::geometry::Size {
                    width: width.maybe_min(size.width),
                    height: (*font_size as f32).maybe_min(size.height),
                })
            }
            Self::Image { path, .. } => {
                let mut image = std::fs::File::open(&path).unwrap();
                let img = cairo::ImageSurface::create_from_png(&mut image).unwrap();
                let width = img.get_width() as f32;
                let height = img.get_height() as f32;
                Ok(stretch::geometry::Size {
                    width: width.maybe_min(size.width),
                    height: height.maybe_min(size.height),
                })
            }
            _ => Err(Box::new(())),
        }
    }
}

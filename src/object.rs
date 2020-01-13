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
        children: Vec<Object>,
        style: S,
        #[serde(default, with = "opt_external_color")]
        background: Option<piet::Color>,
    },
    Text {
        font: String,
        text: String,
        font_size: f64,
        #[serde(with = "ColorDef")]
        color: piet::Color,
        style: S,
    },
}

impl Object {
    pub fn get_style(&self) -> stretch::style::Style {
        match self {
            Self::Container { style, .. } => style.clone(),
            Self::Text { style, .. } => style.clone(),
        }
    }

    pub fn get_background(&self) -> Option<&piet::Color> {
        match self {
            Self::Container { background, .. } => background.as_ref(),
            Self::Text { .. } => None,
        }
    }

    pub fn compute_size(
        &self,
        _size: stretch::geometry::Size<stretch::number::Number>,
    ) -> Result<stretch::geometry::Size<f32>, Box<dyn std::any::Any>> {
        use piet::{FontBuilder, Text, TextLayout, TextLayoutBuilder};
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
                    width,
                    height: *font_size as f32,
                })
            }
            _ => Err(Box::new(())),
        }
    }
}

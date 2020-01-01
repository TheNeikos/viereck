pub fn draw_rectangle(
    rc: &mut impl piet::RenderContext,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: &piet::Color,
) {
    let rect = kurbo::Rect::from_origin_size((x.into(), y.into()), (width.into(), height.into()));

    rc.fill(rect, color);
}

pub fn root_style() -> stretch::style::Style {
    stretch::style::Style {
        size: stretch::geometry::Size {
            width: stretch::style::Dimension::Percent(1.0),
            height: stretch::style::Dimension::Percent(1.0),
        },
        align_content: stretch::style::AlignContent::Stretch,
        ..Default::default()
    }
}

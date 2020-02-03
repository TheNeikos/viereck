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

pub fn draw_rounded_rectangle(
    rc: &mut impl piet::RenderContext,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    radius: f64,
    color: &piet::Color,
) {
    let rect = kurbo::RoundedRect::from_origin_size(
        (x as f64, y as f64).into(),
        (width as f64, height as f64).into(),
        radius as f64,
    );

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

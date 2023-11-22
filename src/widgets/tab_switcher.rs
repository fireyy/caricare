pub fn render_switch(
    &self,
    ui: &mut Ui,
    id: Id,
    labels: &[&str; 2],
    selected: usize,
    available: Rect,
) -> Response {
    let text_color = ui.visuals().strong_text_color();
    let galley1 = ui.fonts(|fonts| {
        fonts.layout_no_wrap(labels[0].to_owned(), FontId::monospace(9.0), text_color)
    });
    let galley2 = ui.fonts(|fonts| {
        fonts.layout_no_wrap(labels[1].to_owned(), FontId::monospace(9.0), text_color)
    });

    let margin = vec2(10.0, 4.0);

    // Initial estimation of the position. It will be moved below
    let rect = Rect::from_min_size(
        Pos2::ZERO,
        vec2(
            margin.x + galley1.rect.width() + margin.x + galley2.rect.width() + margin.x,
            margin.y * 2.0 + galley1.rect.height(),
        ),
    );
    let split = (margin.x + galley1.rect.width() + margin.x * 0.5) / rect.width();
    let rect = rect.translate((available.right_top() + vec2(-(rect.width() + 4.0), 6.0)).to_vec2());
    let response = ui.interact(rect, id, Sense::click());

    let duration = if response.hovered() { 1.0 } else { 2.0 };
    let how_hovered = ui.ctx().animate_bool_with_time(
        id.with("hover"),
        response.hovered(),
        ui.style().animation_time * duration,
    );
    let (mut left, mut right) = rect.split_left_right_at_fraction(split);

    fn lerp(a: f32, b: f32, amt: f32) -> f32 {
        a + (b - a) * amt
    }

    // Collapse the side that is not selected
    if selected == 1 {
        left.set_left(lerp(right.left(), left.left(), how_hovered));
    } else {
        right.set_right(lerp(left.right(), right.right(), how_hovered));
        left.set_right(lerp(
            left.right() + margin.x * 0.5,
            left.right(),
            how_hovered,
        ));
    }
    let rect = left.union(right);

    // Adjust all rects to stay right-aligned
    let adjustment = vec2(available.right() - rect.right() - 4.0, 0.0);
    let rect = rect.translate(adjustment);
    let left = left.translate(adjustment);
    let right = right.translate(adjustment);

    let bg_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
    let bg_opacity = how_hovered.powf(3.0);

    // Background
    ui.painter().rect(
        rect,
        Rounding::same(rect.height() * 0.5),
        ui.visuals().window_fill(),
        Stroke::new(1.0, bg_color),
    );

    // Animates between 0.0 and 1.0
    let amt = ui.ctx().animate_value_with_time(id, selected as f32, 0.1);

    // Selection rect
    let radius = left.height() * 0.5;
    let left_rounding = Rounding {
        nw: radius,
        ne: 2.0,
        se: 2.0,
        sw: radius,
    };
    let right_rounding = Rounding {
        nw: 2.0,
        ne: radius,
        se: radius,
        sw: 2.0,
    };
    let select_rounding = left_rounding
        .lerp_towards(&right_rounding, amt)
        .lerp_towards(&Rounding::same(radius), 1.0 - how_hovered);
    let select_rect = left.lerp_towards(&right, amt);
    ui.painter().rect_filled(
        select_rect.shrink(2.0),
        select_rounding,
        bg_color.gamma_multiply(bg_opacity),
    );

    // Right text
    let mut painter = ui.painter().clone();
    painter.set_clip_rect(right);
    painter.galley(right.center() - galley2.rect.max.to_vec2() * 0.5, galley2);

    // Left text
    let mut painter = ui.painter().clone();
    painter.set_clip_rect(left);
    painter.galley(left.center() - galley1.rect.max.to_vec2() * 0.5, galley1);

    if response.hovered() {
        ui.output_mut(|out| out.cursor_icon = CursorIcon::PointingHand);
    }
    response
}

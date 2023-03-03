use crate::state::State;
use egui::Vec2;

#[derive(PartialEq)]
pub enum ZoomType {
    In,
    Out,
}

pub fn zoomratio(i: f32, s: f32) -> f32 {
    i * s * 0.1
}

// let delta = zoomratio(delta_y, state.scale);
// let new_scale = state.scale + delta;
// // limit scale
// if new_scale > 0.01 && new_scale < 40. {
//     state.offset -= scale_pt(state.offset, state.cursor, state.scale, delta);
//     state.scale += delta;
// }

pub fn scale_pt(origin: Vec2, pt: Vec2, scale: f32, scale_inc: f32) -> Vec2 {
    ((pt - origin) * scale_inc) / scale
}

fn zoom_action(win_size: egui::Vec2, state: &mut State, zoom_type: ZoomType) {
    let i = if zoom_type == ZoomType::In { 3.5 } else { -3.5 };
    let delta = zoomratio(i, state.img_zoom);
    let new_scale = state.img_zoom + delta;
    // limit scale
    if new_scale > 0.05 && new_scale < 40. {
        // We want to zoom towards the center
        let center = Vec2::new(win_size.x as f32 / 2., win_size.y as f32 / 2.);
        state.offset -= scale_pt(state.offset, center, state.img_zoom, delta);
        state.img_zoom += delta;
    }
}

pub fn image_view_ui(ctx: &egui::Context, state: &mut State) {
    let url = state.get_oss_url(&state.current_img.path);

    if url.is_empty() {
        return;
    }

    let win_size = ctx.input(|i| i.screen_rect).size();
    let frame = egui::Frame {
        fill: ctx.style().visuals.panel_fill,
        ..state.cc_ui.bottom_panel_frame()
    };
    egui::SidePanel::right("preview_panel")
        .default_width(100.0)
        .resizable(true)
        .frame(frame)
        .show_animated(ctx, state.is_preview, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false; 2])
                .max_height(win_size.y - 100.0)
                .show(ui, |ui| {
                    if let Some(img) = state.images.get(&url) {
                        let mut size = img.size_vec2();
                        size = if state.img_zoom != 1.0 {
                            size * state.img_zoom
                        } else {
                            size * (ui.available_width() / size.x).min(1.0)
                        };
                        img.show_size(ui, size);
                    }
                });
            ui.vertical_centered_justified(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("\u{2795}").on_hover_text("Zoom In").clicked() {
                        zoom_action(win_size, state, ZoomType::In);
                    }
                    if ui
                        .button("\u{1f5d6}")
                        .on_hover_text("Zoom to window size")
                        .clicked()
                    {
                        state.img_zoom = 1.0;
                    }
                    if ui.button("\u{2796}").on_hover_text("Zoom Out").clicked() {
                        zoom_action(win_size, state, ZoomType::Out);
                    }
                });
                let mut url = url;
                let resp = ui.add(egui::TextEdit::singleline(&mut url));
                if resp.on_hover_text("Click to copy").clicked() {
                    ui.output_mut(|o| o.copied_text = url);
                }
                ui.horizontal(|ui| {
                    ui.label(format!("size: {}", state.current_img.size));
                    ui.label(&state.current_img.last_modified);
                });
            });
        });
}

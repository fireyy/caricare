use crate::state::State;

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
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .max_height(win_size.y - 80.0)
                .show(ui, |ui| {
                    if let Some(img) = state.images.get(&url) {
                        let mut size = img.size_vec2();
                        size *= (ui.available_width() / size.x).min(1.0);
                        img.show_size(ui, size);
                    }
                });
            ui.vertical_centered_justified(|ui| {
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

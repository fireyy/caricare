use crate::state::State;

pub fn log_panel_ui(ctx: &egui::Context, state: &mut State) {
    let frame = egui::Frame {
        fill: ctx.style().visuals.panel_fill,
        ..state.cc_ui.bottom_panel_frame()
    };
    egui::TopBottomPanel::bottom("result_panel")
        .default_height(100.0)
        .resizable(true)
        .frame(frame)
        .show_animated(ctx, state.is_show_result, |ui| {
            egui::Frame::none()
                .fill(ui.style().visuals.extreme_bg_color)
                .inner_margin(ui.style().spacing.window_margin)
                .show(ui, |ui| {
                    let row_height = ui.text_style_height(&egui::TextStyle::Body);
                    egui::ScrollArea::vertical()
                        .id_source("scroller_logs")
                        .auto_shrink([false; 2])
                        .show_rows(ui, row_height, state.logs.len(), |ui, row_range| {
                            // tracing::info!("row_range: {:?}", row_range);
                            for i in row_range {
                                if let Some(data) = state.logs.get(i) {
                                    ui.label(egui::RichText::new(format!(
                                        "{:?}: {}",
                                        data.log_type, data.data
                                    )));
                                }
                            }
                        });
                });
        });
}

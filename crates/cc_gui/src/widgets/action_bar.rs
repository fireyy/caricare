use crate::state::State;

pub fn action_bar_ui(ctx: &egui::Context, state: &mut State) {
    let selected: Vec<&cc_oss::prelude::Object> =
        state.list.iter().filter(|x| x.selected).collect();
    let frame = egui::Frame {
        fill: ctx.style().visuals.panel_fill,
        ..state.cc_ui.bottom_panel_frame()
    };
    egui::TopBottomPanel::bottom("action_panel")
        .default_height(50.0)
        .resizable(false)
        .frame(frame)
        .show_animated(ctx, !selected.is_empty(), |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Selected: {}", selected.len()));
            });
        });
}

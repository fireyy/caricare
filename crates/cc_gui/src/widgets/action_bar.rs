use crate::state::State;

pub fn action_bar_ui(ctx: &egui::Context, state: &mut State) {
    let has_selected = state.list.iter().find(|x| x.selected());
    let frame = egui::Frame {
        fill: ctx.style().visuals.panel_fill,
        ..state.cc_ui.bottom_panel_frame()
    };
    egui::TopBottomPanel::bottom("action_panel")
        .default_height(50.0)
        .resizable(false)
        .frame(frame)
        .show_animated(ctx, has_selected.is_some(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Action:");
                if ui.button("Copy").clicked() {
                    //
                }
                if ui.button("Move").clicked() {
                    //
                }
                if ui.button("Rename").clicked() {
                    //
                }
                // if ui.button("Delete").clicked() {
                //     //
                // }
            });
        });
}

use crate::state::State;
use crate::theme::icon;

pub fn transfer_panel_ui(ctx: &egui::Context, state: &mut State) {
    let frame = egui::Frame {
        fill: ctx.style().visuals.panel_fill,
        ..state.cc_ui.bottom_panel_frame()
    };
    egui::TopBottomPanel::bottom("transfer_panel")
        .default_height(100.0)
        .resizable(true)
        .frame(frame)
        .max_height(300.0)
        .show_animated(ctx, state.is_show_transfer, |ui| {
            egui::Frame::none()
                .inner_margin(ui.style().spacing.window_margin)
                .show(ui, |ui| {
                    ui.columns(2, |ui| {
                        ui[0].text_edit_singleline(&mut state.transfer_filter_str);
                        ui[1].horizontal(|ui| {
                            if ui.button(icon::PAUSE).clicked() {
                                //
                            }
                            if ui.button(icon::DELETE).clicked() {
                                //
                            }
                        });
                    });
                    ui.add_space(10.0);
                    let row_height = ui.text_style_height(&egui::TextStyle::Body);
                    egui::ScrollArea::vertical()
                        .id_source("transfer_scroller")
                        .auto_shrink([false; 2])
                        .always_show_scroll(true)
                        .show_rows(ui, row_height, 5, |ui, row_range| {
                            // tracing::info!("row_range: {:?}", row_range);
                            egui::Grid::new("transfer_grid".to_string())
                                .num_columns(1)
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing.x = 0.0;
                                    for i in row_range {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                egui::Frame::none().show(ui, |ui| {
                                                    ui.set_width(60.);
                                                    ui.spacing_mut().item_spacing.x = 10.0;
                                                    if ui.button(icon::PAUSE).clicked() {
                                                        //
                                                    }
                                                    if ui.button(icon::DELETE).clicked() {
                                                        //
                                                    }
                                                });
                                                egui::Frame::none().show(ui, |ui| {
                                                    ui.set_width(ui.available_width() - 180.0);
                                                    ui.scope(|ui| {
                                                        ui.spacing_mut().interact_size.y = 8.0;
                                                        ui.add(
                                                            egui::ProgressBar::new(40.0 / 100.0)
                                                                .show_percentage(),
                                                        );
                                                    });
                                                });
                                                ui.with_layout(
                                                    egui::Layout::left_to_right(egui::Align::TOP),
                                                    |ui| {
                                                        ui.set_width(120.);
                                                        ui.vertical(|ui| {
                                                            ui.add(egui::Label::new(
                                                                state.cc_ui.text_ellipsis(
                                                                    &format!("{i}-1234567890.png"),
                                                                    1,
                                                                ),
                                                            ));
                                                        });
                                                    },
                                                );
                                            },
                                        );
                                        ui.end_row();
                                    }
                                });
                        });
                });
        });
}

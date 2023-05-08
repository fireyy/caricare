use crate::global;
use crate::state::State;
use cc_ui::icon;

pub fn transfer_panel_ui(ctx: &egui::Context, state: &mut State) {
    let frame = egui::Frame {
        fill: ctx.style().visuals.panel_fill,
        ..global().cc_ui.bottom_panel_frame()
    };
    egui::TopBottomPanel::bottom("transfer_panel")
        .default_height(100.0)
        .resizable(true)
        .frame(frame)
        .max_height(300.0)
        .show_animated(ctx, state.transfer_manager.is_show, |ui| {
            egui::Frame::none()
                .inner_margin(ui.style().spacing.window_margin)
                .show(ui, |ui| {
                    let row_height = ui.text_style_height(&egui::TextStyle::Body);
                    let data = state.transfer_manager.data().clone();
                    let row_size = data.len();
                    // ui.columns(2, |ui| {
                    //     ui[0].text_edit_singleline(&mut state.transfer_manager.filter);
                    //     ui[1].horizontal(|ui| {
                    //         if ui.button(icon::DELETE).clicked() {
                    //             //
                    //         }
                    //     });
                    // });
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut state.transfer_manager.t_type,
                            "download".into(),
                            egui::RichText::new("Download"),
                        );
                        ui.selectable_value(
                            &mut state.transfer_manager.t_type,
                            "upload".into(),
                            egui::RichText::new("Upload"),
                        );
                    });
                    ui.add_space(10.0);
                    egui::ScrollArea::vertical()
                        .id_source("transfer_scroller")
                        .auto_shrink([false; 2])
                        .always_show_scroll(true)
                        .show_rows(ui, row_height, row_size, |ui, row_range| {
                            // tracing::info!("row_range: {:?}", row_range);
                            egui::Grid::new("transfer_grid".to_string())
                                .num_columns(1)
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing.x = 0.0;
                                    for i in row_range {
                                        let d = data.iter().nth(i).unwrap();
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                egui::Frame::none().show(ui, |ui| {
                                                    ui.set_width(60.);
                                                    ui.spacing_mut().item_spacing.x = 10.0;
                                                    if ui.button(icon::DELETE).clicked() {
                                                        //
                                                    }
                                                });
                                                egui::Frame::none().show(ui, |ui| {
                                                    ui.set_width(ui.available_width() - 180.0);
                                                    ui.scope(|ui| {
                                                        ui.spacing_mut().interact_size.y = 8.0;
                                                        ui.add(
                                                            egui::ProgressBar::new(d.1.rate())
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
                                                                global()
                                                                    .cc_ui
                                                                    .text_ellipsis(d.0, 1),
                                                            ))
                                                            .on_hover_text(d.0);
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

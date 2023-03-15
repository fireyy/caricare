use crate::state::{State, Status};
use crate::widgets::{
    list::{list_ui, thumb_ui},
    status_bar_ui, top_bar_ui,
};
use crate::{THUMB_LIST_HEIGHT, THUMB_LIST_WIDTH};
use cc_core::ShowType;
use egui::NumExt;

pub fn main_page(ctx: &egui::Context, state: &mut State, frame: &mut eframe::Frame) {
    top_bar_ui(ctx, state, frame);
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::Frame::none()
            .inner_margin(egui::style::Margin::same(0.0))
            .show(ui, |ui| {
                if let Status::Busy(_) = state.status {
                    ui.centered_and_justified(|ui| {
                        ui.spinner();
                    });
                    return;
                }
                if let Some(err) = &state.err {
                    state.toasts.error(err);
                    state.err = None;
                }
                let list_len = state.list.len();
                if state.list.is_empty() {
                    ui.centered_and_justified(|ui| ui.heading("Nothing Here."));
                    return;
                }
                let (num_cols, num_rows, col_width, row_height) = match state.setting.show_type {
                    ShowType::List => (
                        1,
                        list_len,
                        1.0,
                        ui.text_style_height(&egui::TextStyle::Body),
                    ),
                    ShowType::Thumb => {
                        let w = ui.ctx().input(|i| i.screen_rect().size());
                        let num_cols = (w.x / THUMB_LIST_WIDTH) as usize;
                        let num_rows = (list_len as f32 / num_cols as f32).ceil() as usize;
                        let col_width = w.x / (num_cols as f32);

                        (num_cols, num_rows, col_width, THUMB_LIST_HEIGHT)
                    }
                };

                let mut scroller = egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .always_show_scroll(true)
                    // .max_height(200.0)
                    // .enable_scrolling(false)
                    // .hscroll(self.show_type == ShowType::List)
                    .id_source("scroller_".to_owned() + &row_height.to_string());

                if state.scroll_top {
                    state.scroll_top = false;
                    scroller = scroller.scroll_offset(egui::Vec2::ZERO);
                }

                // let response = scroller.show_rows(ui, row_height, num_rows, |ui, row_range| {
                let spacing = ui.spacing().item_spacing;
                let row_height_with_spacing = row_height + spacing.y;
                let (current_scroll, max_scroll) = scroller
                    .show_viewport(ui, |ui, viewport| {
                        // tracing::info!("row_range: {:?}", row_range);
                        ui.set_height(
                            (row_height_with_spacing * num_rows as f32 - spacing.y).at_least(0.0),
                        );

                        let mut min_row =
                            (viewport.min.y / row_height_with_spacing).floor() as usize;
                        let mut max_row =
                            (viewport.max.y / row_height_with_spacing).ceil() as usize + 1;
                        if max_row > num_rows {
                            let diff = max_row.saturating_sub(min_row);
                            max_row = num_rows;
                            min_row = num_rows.saturating_sub(diff);
                        }

                        let y_min = ui.max_rect().top() + min_row as f32 * row_height_with_spacing;
                        let y_max = ui.max_rect().top() + max_row as f32 * row_height_with_spacing;

                        let rect =
                            egui::Rect::from_x_y_ranges(ui.max_rect().x_range(), y_min..=y_max);

                        ui.allocate_ui_at_rect(rect, |viewport_ui| {
                            viewport_ui.skip_ahead_auto_ids(min_row); // Make sure we get consistent IDs.
                            match state.setting.show_type {
                                ShowType::List => list_ui(state, viewport_ui, min_row..max_row),
                                ShowType::Thumb => thumb_ui(
                                    state,
                                    viewport_ui,
                                    min_row..max_row,
                                    num_cols,
                                    col_width,
                                ),
                            };
                        });

                        let margin = ui.visuals().clip_rect_margin;
                        let current_scroll = ui.clip_rect().top() - ui.min_rect().top() + margin;
                        let max_scroll =
                            ui.min_rect().height() - ui.clip_rect().height() + 2.0 * margin;

                        (current_scroll, max_scroll)
                    })
                    .inner;

                // tracing::info!(
                //     "Scroll offset: {:.0}, max_scroll: {:.0}",
                //     current_scroll,
                //     max_scroll,
                // );

                if state.next_query.is_some()
                    && current_scroll + 20.0 >= max_scroll
                    && !state.loading_more
                {
                    state.loading_more = true;
                    state.load_more();
                }
            });
    });
    status_bar_ui(ctx, state, frame);
}

use crate::state::State;
use crate::widgets::{
    list::{list_ui, thumb_ui},
    status_bar_ui, top_bar_ui,
};
use crate::{THUMB_LIST_HEIGHT, THUMB_LIST_WIDTH};
use cc_core::ShowType;

pub fn main_page(ctx: &egui::Context, state: &mut State) {
    top_bar_ui(ctx, state);
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::Frame::none()
            .inner_margin(egui::style::Margin::same(0.0))
            .show(ui, |ui| {
                if let Some(err) = &state.err {
                    ui.centered_and_justified(|ui| {
                        ui.label(egui::RichText::new(err).color(egui::Color32::RED))
                    });
                    return;
                }
                if state.list.is_empty() {
                    ui.centered_and_justified(|ui| ui.heading("Nothing Here."));
                    return;
                }
                let (num_cols, num_rows, col_width, row_height) = match state.setting.show_type {
                    ShowType::List => (
                        1,
                        state.list.len(),
                        1.0,
                        ui.text_style_height(&egui::TextStyle::Body),
                    ),
                    ShowType::Thumb => {
                        let w = ui.ctx().input(|i| i.screen_rect().size());
                        let num_cols = (w.x / THUMB_LIST_WIDTH) as usize;
                        let num_rows = (state.list.len() as f32 / num_cols as f32).ceil() as usize;
                        let col_width = w.x / (num_cols as f32);

                        (num_cols, num_rows, col_width, THUMB_LIST_HEIGHT)
                    }
                };

                let mut scroller = egui::ScrollArea::vertical()
                    .id_source("scroller_".to_owned() + &row_height.to_string())
                    .auto_shrink([false; 2])
                    // .enable_scrolling(false)
                    // .hscroll(self.show_type == ShowType::List)
                    .id_source("content_scroll");

                if state.scroll_top {
                    state.scroll_top = false;
                    scroller = scroller.scroll_offset(egui::Vec2::ZERO);
                }

                let (current_scroll, max_scroll) = scroller
                    .show_rows(ui, row_height, num_rows, |ui, row_range| {
                        // tracing::info!("row_range: {:?}", row_range);
                        match state.setting.show_type {
                            ShowType::List => list_ui(state, ui, row_range),
                            ShowType::Thumb => thumb_ui(state, ui, row_range, num_cols, col_width),
                        }
                        let margin = ui.visuals().clip_rect_margin;
                        let current_scroll = ui.clip_rect().top() - ui.min_rect().top() + margin;
                        let max_scroll =
                            ui.min_rect().height() - ui.clip_rect().height() + 2.0 * margin;
                        (current_scroll, max_scroll)
                    })
                    .inner;

                // tracing::info!(
                //     "current_scroll: {}, max_scroll: {}",
                //     current_scroll,
                //     max_scroll
                // );

                if state.next_query.is_some() && current_scroll >= max_scroll && !state.loading_more
                {
                    state.loading_more = true;
                    state.load_more(ui.ctx());
                }
            });
    });
    status_bar_ui(ctx, state);
}

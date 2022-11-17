use eframe::epaint::text::{LayoutJob, TextWrapping};
use egui::{Align, Color32, TextFormat};

pub fn text_ellipsis(name: &str, max_rows: usize) -> LayoutJob {
    let mut job = LayoutJob::single_section(
        name.to_string(),
        TextFormat {
            color: Color32::BLACK,
            valign: Align::Center,
            ..TextFormat::default()
        },
    );

    job.wrap = TextWrapping {
        max_rows,
        break_anywhere: true,
        overflow_character: Some('…'),
        ..TextWrapping::default()
    };

    job
}

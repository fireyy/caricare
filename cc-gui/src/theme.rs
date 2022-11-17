use eframe::epaint::text::{LayoutJob, TextWrapping};
use egui::{Align, Color32, FontId, TextFormat};

pub fn text_ellipsis(name: &str, max_rows: usize) -> LayoutJob {
    let mut job = LayoutJob::single_section(
        name.to_string(),
        TextFormat {
            color: Color32::from_gray(100),
            valign: Align::Center,
            font_id: FontId::monospace(14.0),
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

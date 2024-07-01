mod design_tokens;
pub mod icon;

use design_tokens::DesignTokens;
use eframe::epaint::text::{LayoutJob, TextWrapping};

pub const FULLSIZE_CONTENT: bool = cfg!(target_os = "macos");

pub const CUSTOM_WINDOW_DECORATIONS: bool = false;

pub static THUMB_LIST_WIDTH: f32 = 200.0;
pub static THUMB_LIST_HEIGHT: f32 = 50.0;

pub struct TopBarStyle {
    /// Height of the top bar
    pub height: f32,

    /// Extra horizontal space in the top left corner to make room for
    /// close/minimize/maximize buttons (on Mac)
    pub indent: f32,
}

#[derive(Clone, Debug)]
pub struct CCUi {
    pub egui_ctx: egui::Context,
    pub design_tokens: DesignTokens,
}

impl CCUi {
    /// Create [`CCUi`] and apply style to the given egui context.
    pub fn load_and_apply(egui_ctx: &egui::Context) -> Self {
        Self {
            egui_ctx: egui_ctx.clone(),
            design_tokens: DesignTokens::load_and_apply(egui_ctx),
        }
    }

    /// Margin on all sides of views.
    pub fn view_padding() -> f32 {
        12.0
    }

    pub fn window_rounding() -> f32 {
        12.0
    }

    pub fn normal_rounding() -> f32 {
        6.0
    }

    pub fn small_rounding() -> f32 {
        4.0
    }

    pub fn table_line_height() -> f32 {
        14.0
    }

    pub fn table_header_height() -> f32 {
        20.0
    }

    pub fn top_bar_margin() -> egui::Margin {
        egui::Margin::symmetric(8.0, 2.0)
    }

    /// Height of the top-most bar.
    pub fn top_bar_height() -> f32 {
        44.0
    }

    pub fn native_window_rounding() -> f32 {
        10.0
    }

    pub fn top_panel_frame(&self) -> egui::Frame {
        let mut frame = egui::Frame {
            inner_margin: Self::top_bar_margin(),
            fill: self.design_tokens.top_bar_color,
            ..Default::default()
        };
        if CUSTOM_WINDOW_DECORATIONS {
            frame.rounding.nw = Self::native_window_rounding();
            frame.rounding.ne = Self::native_window_rounding();
        }
        frame
    }

    pub fn bottom_panel_margin(&self) -> egui::Vec2 {
        egui::Vec2::splat(8.0)
    }

    /// For the streams view (time panel)
    pub fn bottom_panel_frame(&self) -> egui::Frame {
        // Show a stroke only on the top. To achieve this, we add a negative outer margin.
        // (on the inner margin we counteract this again)
        let margin_offset = self.design_tokens.bottom_bar_stroke.width * 0.5;

        let margin = self.bottom_panel_margin();

        let mut frame = egui::Frame {
            fill: self.design_tokens.bottom_bar_color,
            inner_margin: egui::Margin::symmetric(
                margin.x + margin_offset,
                margin.y + margin_offset,
            ),
            outer_margin: egui::Margin {
                left: -margin_offset,
                right: -margin_offset,
                // Add a proper stoke width thick margin on the top.
                top: self.design_tokens.bottom_bar_stroke.width,
                bottom: -margin_offset,
            },
            stroke: self.design_tokens.bottom_bar_stroke,
            rounding: self.design_tokens.bottom_bar_rounding,
            ..Default::default()
        };
        if CUSTOM_WINDOW_DECORATIONS {
            frame.rounding.sw = Self::native_window_rounding();
            frame.rounding.se = Self::native_window_rounding();
        }
        frame
    }

    pub fn top_bar_style(&self, style_like_web: bool) -> TopBarStyle {
        let egui_zoom_factor = self.egui_ctx.zoom_factor();

        let fullscreen = self
            .egui_ctx
            .input(|i| i.viewport().fullscreen)
            .unwrap_or(false);

        // On Mac, we share the same space as the native red/yellow/green close/minimize/maximize buttons.
        // This means we need to make room for them.
        let make_room_for_window_buttons = !style_like_web && {
            #[cfg(target_os = "macos")]
            {
                crate::FULLSIZE_CONTENT && !fullscreen
            }
            #[cfg(not(target_os = "macos"))]
            {
                _ = fullscreen;
                false
            }
        };

        let native_buttons_size_in_native_scale = egui::vec2(64.0, 24.0); // source: I measured /emilk

        let height = if make_room_for_window_buttons {
            // On mac we want to match the height of the native red/yellow/green close/minimize/maximize buttons.
            // TODO(emilk): move the native window buttons to match our Self::title_bar_height

            // Use more vertical space when zoomed in…
            let height = native_buttons_size_in_native_scale.y;

            // …but never shrink below the native button height when zoomed out.
            height.max(native_buttons_size_in_native_scale.y / egui_zoom_factor)
        } else {
            Self::top_bar_height() - Self::top_bar_margin().sum().y
        };

        let indent = if make_room_for_window_buttons {
            // Always use the same width measured in native GUI coordinates:
            native_buttons_size_in_native_scale.x / egui_zoom_factor
        } else {
            0.0
        };

        TopBarStyle { height, indent }
    }

    pub fn text_ellipsis(&self, name: &str, max_rows: usize) -> LayoutJob {
        text_ellipsis(&self.egui_ctx.style(), name, max_rows)
    }
}

pub fn text_ellipsis(style: &egui::Style, name: &str, max_rows: usize) -> LayoutJob {
    let font_id = egui::TextStyle::Body.resolve(style);
    let color = style.visuals.text_color();
    let mut job =
        LayoutJob::single_section(name.to_string(), egui::TextFormat::simple(font_id, color));

    job.wrap = TextWrapping {
        max_rows,
        break_anywhere: true,
        overflow_character: Some('…'),
        ..TextWrapping::default()
    };

    job
}

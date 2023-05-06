//! Adapted from https://github.com/rerun-io/rerun/blob/main/crates/re_ui/src/design_tokens.rs
use egui::Color32;

/// The look and feel of the UI.
///
/// Not everything is covered by this.
/// A lot of other design tokens are put straight into the [`egui::Style`]
#[derive(Clone, Copy, Debug)]
pub struct DesignTokens {
    pub top_bar_color: egui::Color32,
    pub bottom_bar_color: egui::Color32,
    pub bottom_bar_stroke: egui::Stroke,
    pub bottom_bar_rounding: egui::Rounding,
    pub shadow_gradient_dark_start: egui::Color32,
    pub selection_color: egui::Color32,
}

impl DesignTokens {
    /// Create [`DesignTokens`] and apply style to the given egui context.
    pub fn load_and_apply(ctx: &egui::Context) -> Self {
        apply_design_tokens(ctx)
    }
}

fn apply_design_tokens(ctx: &egui::Context) -> DesignTokens {
    let apply_font = true;
    let apply_font_size = true;

    if apply_font {
        let mut font_definitions = egui::FontDefinitions::default();
        font_definitions.font_data.insert(
            "Inter-Medium".into(),
            egui::FontData::from_static(include_bytes!("./data/Inter-Medium.otf")),
        );
        // icon font
        font_definitions.font_data.insert(
            "Icon".into(),
            egui::FontData::from_static(include_bytes!("./data/icon.ttf")).tweak(egui::FontTweak {
                scale: 1.0,
                y_offset_factor: 0.0,
                y_offset: 0.0,
            }),
        );

        font_definitions
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "Inter-Medium".into());

        // chinese font
        if cfg!(feature = "lang-cjk") {
            font_definitions.font_data.insert(
                "SourceHan-Medium".into(),
                egui::FontData::from_static(include_bytes!("./data/SourceHanSansCN-Medium.otf")),
            );

            font_definitions
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(1, "SourceHan-Medium".into());

            font_definitions
                .families
                .get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .push("SourceHan-Medium".into());
        }

        font_definitions
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(2, "Icon".into());

        ctx.set_fonts(font_definitions);
    }

    let mut egui_style = egui::Style {
        visuals: egui::Visuals::dark(),
        ..Default::default()
    };

    if apply_font_size {
        let font_size = 12.0;

        for text_style in [
            egui::TextStyle::Body,
            egui::TextStyle::Monospace,
            egui::TextStyle::Button,
        ] {
            egui_style.text_styles.get_mut(&text_style).unwrap().size = font_size;
        }

        // We want labels and buttons to have the same height.
        // Intuitively, we would just assign font_size to
        // the interact_size, but in practice text height does not match
        // font size (for unknown reason), so we fudge it for now:

        egui_style.spacing.interact_size.y = 15.0;
        // egui_style.spacing.interact_size.y = font_size;
    }

    let selection_color = Color32::from_rgb(0, 61, 161);
    let panel_bg_color = Color32::from_rgb(13, 16, 17);
    let floating_color = Color32::from_gray(38);

    // Used as the background of text edits, scroll bars and others things
    // that needs to look different from other interactive stuff.
    // We need this very dark, since the theme overall is very, very dark.
    egui_style.visuals.extreme_bg_color = egui::Color32::BLACK;

    egui_style.visuals.widgets.noninteractive.weak_bg_fill = panel_bg_color;
    egui_style.visuals.widgets.noninteractive.bg_fill = panel_bg_color;

    egui_style.visuals.button_frame = true;
    egui_style.visuals.widgets.inactive.weak_bg_fill = Default::default(); // Buttons have no background color when inactive
    egui_style.visuals.widgets.inactive.bg_fill = Color32::from_gray(40);

    {
        // Background colors for buttons (menu buttons, blueprint buttons, etc) when hovered or clicked:
        let hovered_color = Color32::from_gray(64);
        egui_style.visuals.widgets.hovered.weak_bg_fill = hovered_color;
        egui_style.visuals.widgets.hovered.bg_fill = hovered_color;
        egui_style.visuals.widgets.active.weak_bg_fill = hovered_color;
        egui_style.visuals.widgets.active.bg_fill = hovered_color;
        egui_style.visuals.widgets.open.weak_bg_fill = hovered_color;
        egui_style.visuals.widgets.open.bg_fill = hovered_color;
    }

    {
        // Turn off strokes around buttons:
        egui_style.visuals.widgets.inactive.bg_stroke = Default::default();
        egui_style.visuals.widgets.hovered.bg_stroke = Default::default();
        egui_style.visuals.widgets.active.bg_stroke = Default::default();
        egui_style.visuals.widgets.open.bg_stroke = Default::default();
    }

    {
        // Expand hovered and active button frames:
        egui_style.visuals.widgets.hovered.expansion = 2.0;
        egui_style.visuals.widgets.active.expansion = 2.0;
        egui_style.visuals.widgets.open.expansion = 2.0;
    }

    egui_style.visuals.selection.bg_fill = selection_color;

    egui_style.visuals.widgets.noninteractive.bg_stroke.color = Color32::from_gray(30); // from figma. separator lines, panel lines, etc

    let subudued = Color32::from_rgb(125, 140, 146);
    let default = Color32::from_rgb(202, 216, 222);
    let strong = Color32::from_rgb(255, 255, 255);

    egui_style.visuals.widgets.noninteractive.fg_stroke.color = subudued; // non-interactive text
    egui_style.visuals.widgets.inactive.fg_stroke.color = default; // button text
    egui_style.visuals.widgets.active.fg_stroke.color = strong; // strong text and active button text

    egui_style.visuals.popup_shadow = egui::epaint::Shadow::NONE;
    egui_style.visuals.window_shadow = egui::epaint::Shadow::NONE;

    egui_style.visuals.window_fill = floating_color; // tooltips and menus
    egui_style.visuals.window_stroke = egui::Stroke::NONE;
    egui_style.visuals.panel_fill = panel_bg_color;

    egui_style.visuals.window_rounding = super::CCUi::window_rounding().into();
    egui_style.visuals.menu_rounding = super::CCUi::window_rounding().into();
    let small_rounding = super::CCUi::small_rounding().into();
    egui_style.visuals.widgets.noninteractive.rounding = small_rounding;
    egui_style.visuals.widgets.inactive.rounding = small_rounding;
    egui_style.visuals.widgets.hovered.rounding = small_rounding;
    egui_style.visuals.widgets.active.rounding = small_rounding;
    egui_style.visuals.widgets.open.rounding = small_rounding;

    egui_style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    egui_style.spacing.menu_margin = super::CCUi::view_padding().into();

    // Add stripes to grids and tables?
    egui_style.visuals.striped = false;
    egui_style.visuals.indent_has_left_vline = false;
    egui_style.spacing.button_padding = egui::Vec2::new(1.0, 0.0); // Makes the icons in the blueprint panel align
    egui_style.spacing.indent = 14.0; // From figma

    egui_style.debug.show_blocking_widget = false; // turn this on to debug interaction problems

    egui_style.spacing.combo_width = 8.0; // minium width of ComboBox - keep them small, with the down-arrow close.

    egui_style.spacing.scroll_bar_inner_margin = 2.0;
    egui_style.spacing.scroll_bar_width = 6.0;
    egui_style.spacing.scroll_bar_outer_margin = 2.0;

    ctx.set_style(egui_style);

    DesignTokens {
        top_bar_color: Color32::from_gray(20), // copied from figma
        bottom_bar_color: Color32::from_rgb(20, 24, 25),
        bottom_bar_stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(47)), // copied from figma
        bottom_bar_rounding: egui::Rounding {
            nw: super::CCUi::normal_rounding(),
            ne: super::CCUi::normal_rounding(),
            sw: 0.0,
            se: 0.0,
        }, // copied from figma, should be top only
        shadow_gradient_dark_start: egui::Color32::from_black_alpha(77),
        selection_color,
    }
}

// ----------------------------------------------------------------------------

#[test]
fn test_design_tokens() {
    let ctx = egui::Context::default();
    apply_design_tokens(&ctx);

    // Make sure it works:
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello Test!");
        });
    });
}

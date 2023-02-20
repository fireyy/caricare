mod app;
mod state;
mod theme;
mod widgets;

pub use app::App;

pub static SUPPORT_EXTENSIONS: [&str; 4] = ["png", "gif", "jpg", "svg"];
pub static THUMB_LIST_WIDTH: f32 = 200.0;
pub static THUMB_LIST_HEIGHT: f32 = 50.0;

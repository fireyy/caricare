use std::ffi::OsStr;
use std::path::Path;

static SUPPORT_IMG: [&str; 5] = [".png", ".jpg", ".svg", ".gif", ".webp"];

pub fn get_extension(path: &Path) -> String {
    path.extension()
        .and_then(OsStr::to_str)
        .unwrap()
        .to_string()
}

pub fn is_vaild_img(str: &str) -> bool {
    for &a in &SUPPORT_IMG {
        if str.ends_with(a) {
            return true;
        }
    }
    false
}

use std::ffi::OsStr;
use std::path::PathBuf;

#[macro_export]
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

pub fn get_extension(path: &PathBuf) -> String {
    path.extension()
        .and_then(OsStr::to_str)
        .unwrap()
        .to_string()
}

pub fn get_name(path: &PathBuf) -> String {
    path.file_name()
        .and_then(OsStr::to_str)
        .unwrap()
        .to_string()
}

pub fn is_vaild_img(str: &String) -> bool {
    regex!(r"(?i)^(.*)(\.png|\.jpg|\.svg|\.gif)$").is_match(&str)
}

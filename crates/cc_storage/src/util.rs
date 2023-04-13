use std::ffi::OsStr;
use std::path::Path;

pub fn get_name(path: &Path) -> String {
    path.file_name()
        .and_then(OsStr::to_str)
        .unwrap()
        .to_string()
}

pub fn get_name_form_path(path: &str) -> String {
    path.split('/')
        .filter(|k| !k.is_empty())
        .last()
        .unwrap_or("")
        .to_string()
}

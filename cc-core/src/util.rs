use std::ffi::OsStr;
use std::path::PathBuf;

pub fn get_extension(path: PathBuf) -> String {
    path.extension()
        .and_then(OsStr::to_str)
        .unwrap()
        .to_string()
}

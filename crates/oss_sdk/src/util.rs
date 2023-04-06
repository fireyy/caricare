use std::ffi::OsStr;
use std::io::Write;
use std::path::Path;

use once_cell::sync::Lazy;

use crate::Result;

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

pub(crate) fn check_bucket_name(name: &str) -> Result<()> {
    let len = name.len();
    if !(3..=63).contains(&len) {
        anyhow::bail!("bucket name {} len is between [3-63],now is {}", name, &len);
    }
    for ch in name.chars() {
        let valid = ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-';
        if !valid {
            anyhow::bail!(
                "bucket name {} can only include lowercase letters, numbers, and -",
                name
            );
        }
    }

    if name.chars().next().unwrap_or_default() == '-'
        || name.chars().last().unwrap_or_default() == '-'
    {
        anyhow::bail!(
            "bucket name {} must start and end with a lowercase letter or number",
            name
        );
    }
    Ok(())
}

pub(crate) struct SysInfo(String, String, String);

impl SysInfo {
    pub(crate) fn name(&self) -> &str {
        &self.0
    }

    pub(crate) fn release(&self) -> &str {
        &self.1
    }

    pub(crate) fn machine(&self) -> &str {
        &self.2
    }
}

pub(crate) static SYS_INFO: Lazy<SysInfo> = Lazy::new(|| {
    use std::env::consts;
    use std::process::Command;

    let uname = |arg: &str| -> Option<String> {
        match Command::new("uname").arg(arg).output() {
            Ok(output) => {
                if !output.status.success() {
                    return None;
                }

                let mut b = vec![];
                b.write_all(&output.stdout).ok();

                match String::from_utf8(b) {
                    Ok(s) => Some(s.trim().into()),
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    };

    let os = uname("-s").unwrap_or_else(|| consts::OS.into());
    let release = uname("-r").unwrap_or_else(|| "-".into());
    let machine = uname("-m").unwrap_or_else(|| consts::ARCH.into());

    SysInfo(os, release, machine)
});

#[cfg(test)]
mod test_super {
    use super::*;

    #[test]
    fn test_sysinfo() {
        println!(
            "os={}, release={}, machine={}",
            SYS_INFO.name(),
            SYS_INFO.release(),
            SYS_INFO.machine()
        );
    }
}

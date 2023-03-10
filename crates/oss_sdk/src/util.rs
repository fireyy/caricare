use std::io::Write;

use once_cell::sync::Lazy;

use crate::Result;

pub(crate) fn check_bucket_name(name: &str) -> Result<()> {
    let len = name.len();
    if len < 3 || len > 63 {
        return bail!("bucket name {} len is between [3-63],now is {}", name, &len);
    }
    for ch in name.chars() {
        let valid = ('a' <= ch && ch <= 'z') || ('0' <= ch && ch <= '9') || ch == '-';
        if !valid {
            return bail!(
                "bucket name {} can only include lowercase letters, numbers, and -",
                name
            );
        }
    }

    if name.chars().nth(0).unwrap_or_default() == '-'
        || name.chars().last().unwrap_or_default() == '-'
    {
        return bail!(
            "bucket name {} must start and end with a lowercase letter or number",
            name
        );
    }
    Ok(())
}

pub(crate) fn query_escape(input: &str) -> String {
    let s = serde_urlencoded::to_string(vec![("k", input)]).expect("Convert query escape failed!");
    s[2..].replace("+", "%20")
}

pub(crate) fn httptime() -> String {
    format!("{}", chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT"))
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

    #[test]
    fn test_query_escape() {
        println!("{}", query_escape("abc"));
    }
}

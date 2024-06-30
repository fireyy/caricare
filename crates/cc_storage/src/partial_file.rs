use crate::Result;
use anyhow::Context;
use std::{
    ffi::OsString,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

/// Small helper struct to make writing files a bit safer by first writing to a
/// hidden file and once finished renaming it to the requested name.
#[derive(Debug)]
pub struct PartialFile {
    target_path: PathBuf,
    partial_path: PathBuf,
    partial_file: BufWriter<File>,
    finished: bool,
}

impl PartialFile {
    pub fn create(target_path: impl Into<PathBuf>) -> Result<Self> {
        let target_path: PathBuf = target_path.into();
        let partial_path = generate_partial_file_name(&target_path)
            .context("could not generate name for partial/temporary file")?;
        let partial_file = File::create(&partial_path).with_context(|| {
            format!(
                "could not create partial/temp file `{}`",
                partial_path.display()
            )
        })?;
        let partial_file = BufWriter::new(partial_file);

        Ok(PartialFile {
            target_path,
            partial_path,
            partial_file,
            finished: false,
        })
    }

    pub fn finish(mut self) -> Result<File> {
        self.partial_file.flush().with_context(|| {
            format!(
                "failed to flush outstanding writes to `{}`",
                self.partial_path.display()
            )
        })?;
        fs::rename(&self.partial_path, &self.target_path).with_context(|| {
            format!(
                "cannot finish partial file `{}`, renaming it to `{}` failed",
                self.partial_path.display(),
                self.target_path.display()
            )
        })?;
        self.finished = true;
        File::open(&self.target_path)
            .with_context(|| format!("cannot open finished file `{}`", self.target_path.display()))
    }
}

impl Write for PartialFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.partial_file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.partial_file.flush()
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.partial_file.write_vectored(bufs)
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.partial_file.write_all(buf)
    }
}

impl Drop for PartialFile {
    fn drop(&mut self) {
        if self.finished {
            return;
        }

        tracing::info!("Deleting partial file `{}`.", self.partial_path.display());
        tracing::debug!(
            "Partial file `{}` was meant to be moved to `{}` once finished",
            self.partial_path.display(),
            self.target_path.display()
        );
        if let Err(e) = fs::remove_file(&self.partial_path) {
            tracing::warn!(
                "Could not delete partial file `{}`: {}",
                self.partial_path.display(),
                e
            )
        }
    }
}

fn generate_partial_file_name(path: &Path) -> Result<PathBuf> {
    let target_file_name = path
        .file_name()
        .with_context(|| format!("cannot get file name from path `{}`", path.display()))?;
    let temp_prefix = {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .context("cannot get timestamp")?;
        format!("artefacta-temp-{}", timestamp.as_secs())
    };
    let new_file_name = {
        let mut res = OsString::from("._");
        res.push(&temp_prefix);
        res.push(target_file_name);
        res.push(".part");
        res
    };
    Ok(path.with_file_name(new_file_name))
}

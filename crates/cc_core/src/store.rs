use crate::{CoreError, Session};
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Serialize};
use std::path::{Path, PathBuf};

static STORE: Lazy<ContentStore> = Lazy::new(|| ContentStore::default());

pub fn set_local_config<T: Serialize>(name: &str, val: &T) {
    let config_path = STORE.config_dir().join(name);
    let raw = serde_json::to_vec_pretty(val).expect("must be valid serde struct");
    std::fs::write(config_path, raw).expect("failed to write");
}

pub fn get_local_config<T: DeserializeOwned>(name: &str) -> Option<T> {
    let config_path = STORE.config_dir().join(name);
    let raw = std::fs::read(config_path).ok()?;
    serde_json::from_slice(&raw).ok()
}

pub fn get_latest_session() -> Option<Session> {
    get_session_by_path(STORE.latest_session_file())
}

pub fn get_session_by_path(path: &Path) -> Option<Session> {
    let session_raw = std::fs::read(path).ok()?;
    let session = serde_json::from_slice::<Session>(&session_raw)
        .map_err(|err| CoreError::Custom(err.to_string()))
        .ok()?;
    Some(session)
}

pub fn get_all_session() -> Result<Vec<Session>, CoreError> {
    let mut sessions = vec![];
    for entry in std::fs::read_dir(STORE.sessions_dir())? {
        let entry = entry?;
        if entry.file_name() != std::ffi::OsString::from("latest") {
            let path = entry.path();
            if let Some(session) = get_session_by_path(path.as_path()) {
                sessions.push(session);
            }
        }
    }
    Ok(sessions)
}

pub fn put_session(session: &Session) -> Result<(), CoreError> {
    let serialized = serde_json::to_string_pretty(session).expect("failed to serialize");
    let src = STORE.sessions_dir().join(&session.key_id);
    // let _ = std::fs::write(STORE.latest_session_file(), serialized.clone().into_bytes());
    let _ = std::fs::write(&src, serialized.into_bytes());
    std::fs::copy(src, STORE.latest_session_file())?;
    Ok(())
}

pub fn delete_session_by_name(name: &str) {
    let path = STORE.sessions_dir().join(name);
    let _ = std::fs::remove_file(path.as_path());
}

pub fn delete_latest_session() {
    let _ = std::fs::remove_file(STORE.latest_session_file());
}

pub const SESSIONS_DIR_NAME: &str = "sessions";
pub const LOG_FILENAME: &str = "log";
pub const CONTENT_DIR_NAME: &str = "content";
pub const CONFIG_DIR_NAME: &str = "config";

#[derive(Debug, Clone)]
pub struct ContentStore {
    latest_session_file: PathBuf,
    sessions_dir: PathBuf,
    log_file: PathBuf,
    content_dir: PathBuf,
    config_dir: PathBuf,
}

impl Default for ContentStore {
    fn default() -> Self {
        let (sessions_dir, log_file, content_dir, config_dir) =
            match directories_next::ProjectDirs::from("com", "fireyy", "caricare") {
                Some(app_dirs) => (
                    app_dirs.data_dir().join(SESSIONS_DIR_NAME),
                    app_dirs.data_dir().join(LOG_FILENAME),
                    app_dirs.cache_dir().join(CONTENT_DIR_NAME),
                    app_dirs.config_dir().to_path_buf(),
                ),
                // Fallback to current working directory if no HOME is present
                None => (
                    SESSIONS_DIR_NAME.into(),
                    LOG_FILENAME.into(),
                    CONTENT_DIR_NAME.into(),
                    CONFIG_DIR_NAME.into(),
                ),
            };

        Self {
            latest_session_file: sessions_dir.join("latest"),
            sessions_dir,
            log_file,
            content_dir,
            config_dir,
        }
    }
}

impl ContentStore {
    pub fn content_path(&self, id: String) -> PathBuf {
        let normalized_id = urlencoding::encode(id.as_str());
        self.content_dir().join(normalized_id.as_ref())
    }

    pub fn content_exists(&self, id: String) -> bool {
        self.content_path(id).exists()
    }

    pub fn create_req_dirs(&self) -> Result<(), CoreError> {
        use std::fs::create_dir_all;

        create_dir_all(self.content_dir())?;
        create_dir_all(self.sessions_dir())?;
        create_dir_all(self.log_file().parent().unwrap_or_else(|| Path::new(".")))?;
        create_dir_all(self.config_dir())?;

        Ok(())
    }

    #[inline(always)]
    pub fn latest_session_file(&self) -> &Path {
        self.latest_session_file.as_path()
    }

    #[inline(always)]
    pub fn content_dir(&self) -> &Path {
        self.content_dir.as_path()
    }

    #[inline(always)]
    pub fn sessions_dir(&self) -> &Path {
        self.sessions_dir.as_path()
    }

    #[inline(always)]
    pub fn log_file(&self) -> &Path {
        self.log_file.as_path()
    }

    #[inline(always)]
    pub fn config_dir(&self) -> &Path {
        self.config_dir.as_path()
    }
}

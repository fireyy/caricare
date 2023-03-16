#[derive(Default, Debug)]
pub enum LogType {
    Upload,
    Copy,
    Delete,
    #[default]
    Unkown,
}

#[derive(Default)]
pub enum LogState {
    Success,
    Error,
    Warn,
    #[default]
    Info,
}

#[derive(Default)]
pub struct LogItem {
    pub log_type: LogType,
    pub state: LogState,
    pub data: String,
}

impl LogItem {
    pub fn new(log_type: LogType, state: LogState, data: String) -> Self {
        Self {
            log_type,
            state,
            data,
        }
    }

    pub fn upload() -> Self {
        Self {
            log_type: LogType::Upload,
            ..Default::default()
        }
    }

    pub fn copy() -> Self {
        Self {
            log_type: LogType::Copy,
            ..Default::default()
        }
    }

    pub fn delete() -> Self {
        Self {
            log_type: LogType::Delete,
            ..Default::default()
        }
    }

    pub fn unknow() -> Self {
        Self {
            log_type: LogType::Unkown,
            ..Default::default()
        }
    }

    pub fn with_success(mut self, data: String) -> Self {
        self.data = data;
        self
    }

    pub fn with_error(mut self, data: String) -> Self {
        self.data = data;
        self
    }

    pub fn with_info(mut self, data: String) -> Self {
        self.data = data;
        self
    }

    pub fn with_warn(mut self, data: String) -> Self {
        self.data = data;
        self
    }
}

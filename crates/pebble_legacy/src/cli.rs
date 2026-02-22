use std::fmt;

pub const EXIT_OK: i32 = 0;
pub const EXIT_ERROR: i32 = 1;
pub const EXIT_USAGE: i32 = 2;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OutputFormat {
    Human,
    Json,
}

impl OutputFormat {
    pub fn from_json_flag(json: bool) -> Self {
        if json { Self::Json } else { Self::Human }
    }
}

#[derive(Debug)]
pub struct UsageError {
    message: String,
}

impl UsageError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for UsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for UsageError {}

use core::fmt;

#[derive(Debug)]
pub struct Error {
    pub error: ErrorType,
    pub message: String,
}

#[derive(Debug)]
pub enum ErrorType {
    Esp,
    InvalidState,
    Unexpected,
}

impl Error {
    pub fn new_esp(message: &str) -> Self {
        Self {
            error: ErrorType::Esp,
            message: message.to_string(),
        }
    }

    pub fn new_invalid_state(message: &str) -> Self {
        Self {
            error: ErrorType::InvalidState,
            message: message.to_string(),
        }
    }

    pub fn new_unexpected(message: &str) -> Self {
        Self {
            error: ErrorType::Unexpected,
            message: message.to_string(),
        }
    }
}

// 表示用
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {}", self.error, self.message)
    }
}

// std::error::Error を実装（必要なら）
impl std::error::Error for Error {}

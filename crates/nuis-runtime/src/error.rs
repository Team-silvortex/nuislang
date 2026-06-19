use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError(pub String);

impl RuntimeError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for RuntimeError {}

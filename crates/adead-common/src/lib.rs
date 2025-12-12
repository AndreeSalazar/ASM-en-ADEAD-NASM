use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int64,
    String,
    Void,
}

#[derive(Debug, Error)]
pub enum ADeadError {
    #[error("Parse error at {line}:{col}: {message}")]
    ParseError { line: usize, col: usize, message: String },

    #[error("Type error: {message}")]
    TypeError { message: String },

    #[error("Runtime error: {message}")]
    RuntimeError { message: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ADeadError>;


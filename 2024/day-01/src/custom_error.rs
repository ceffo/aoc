use miette::Diagnostic;
use std::fmt::Display;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum AocError {
    #[error(transparent)]
    #[diagnostic(code(aoc::io_error))]
    IoError(#[from] std::io::Error),
    #[diagnostic(code(aoc::parse_error))]
    ParseError(String),
}

impl Display for AocError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AocError::IoError(e) => write!(f, "{}", e),
            AocError::ParseError(e) => write!(f, "{}", e),
        }
    }
}

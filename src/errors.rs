use thiserror::Error;

#[derive(Error, Debug)]
pub enum TheShitError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Security error: {0}")]
    Security(String),
    #[error("Python error: {0}")]
    Python(String),
    #[error("Template error: {0}")]
    Template(String),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
    #[error("PyO3 error: {0}")]
    PyO3(#[from] pyo3::PyErr),
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
}

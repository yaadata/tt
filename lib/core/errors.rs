use thiserror::Error;

#[derive(Error, Debug)]
pub enum FrameworkError {
    #[error("parsing error. details = `{0}`")]
    ParsingError(String),
    #[error("failed to find test methods. details = `{0}`")]
    NotFoundError(String),
    #[error("unknown error occurred. details `{0}`")]
    UnknownError(String),
}

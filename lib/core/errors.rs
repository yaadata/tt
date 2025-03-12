use thiserror::Error;

#[derive(PartialEq, Error, Debug)]
pub enum FrameworkError {
    #[error("parsing error. details = `{0}`")]
    ParsingError(String),
    #[error("failed to find test methods. details = `{0}`")]
    NotFoundError(String),
    #[error("unknown error occurred. details `{0}`")]
    UnknownError(String),
    #[error("precondition error. details = `{0}`")]
    PreconditionError(String),
}

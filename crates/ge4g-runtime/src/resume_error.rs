//! Typed resume classification; validation and candidate commit remain authoritative.
use ge4g_core::Error;

#[derive(Debug)]
pub enum ResumeError {
    ContentRevisionMismatch,
    Invalid(Error),
}
impl ResumeError {
    pub fn code(&self) -> Option<&'static str> {
        match self {
            Self::ContentRevisionMismatch => Some("save_content_revision_mismatch"),
            Self::Invalid(_) => None,
        }
    }
}
impl From<Error> for ResumeError {
    fn from(value: Error) -> Self {
        Self::Invalid(value)
    }
}
impl From<ResumeError> for Error {
    fn from(value: ResumeError) -> Self {
        match value {
            ResumeError::ContentRevisionMismatch => Error("save content revision mismatch".into()),
            ResumeError::Invalid(error) => error,
        }
    }
}

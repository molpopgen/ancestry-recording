use thiserror::Error;

#[derive(Error, Debug)]
pub enum InlineAncestryError {
    #[error("intervals error")]
    IntervalsError,
}

use crate::HalfOpenInterval;
use crate::LargeSignedInteger;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InlineAncestryError {
    #[error("intervals error")]
    IntervalsError, // NOTE: this is a bad name
    #[error("invalid position: {p:?}")]
    InvalidPosition { p: LargeSignedInteger },
    #[error("invalid segment: [{left:?}, {right:?})")]
    InvalidSegment {
        left: LargeSignedInteger,
        right: LargeSignedInteger,
    },
}

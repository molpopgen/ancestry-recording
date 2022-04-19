use crate::LargeSignedInteger;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InlineAncestryError {
    #[error("intervals error")]
    IntervalsError, // NOTE: this is a bad name
    #[error("invalid position: {p:?}")]
    InvalidPosition { p: LargeSignedInteger },
    #[error("invalid genome length: {l:?}")]
    InvalidGenomeLength { l: LargeSignedInteger },
    #[error("invalid segment: [{left:?}, {right:?})")]
    InvalidSegment {
        left: LargeSignedInteger,
        right: LargeSignedInteger,
    },
    #[error("child birth time must be > parent birth time, got {child:?}, {parent:?}")]
    InvalidBirthTimeOrder {
        parent: LargeSignedInteger,
        child: LargeSignedInteger,
    },
}

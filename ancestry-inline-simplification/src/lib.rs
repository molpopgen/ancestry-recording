pub use ancestry_common::{LargeSignedInteger, SignedInteger};

mod ancestry_overlapper;
mod individual_heap;
mod segments;
mod flags;
mod error;

pub(crate) use ancestry_overlapper::AncestryOverlapper;
pub(crate) use segments::*;

pub mod individual;
pub mod population;

// Public API
pub use flags::NodeFlags;
pub use error::InlineAncestryError;

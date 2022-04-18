pub use ancestry_common::{LargeSignedInteger, SignedInteger};

mod ancestry_overlapper;
mod error;
mod flags;
mod individual_heap;
mod segments;
mod util;

pub(crate) use ancestry_overlapper::AncestryOverlapper;
pub(crate) use segments::*;

pub mod individual;
pub mod population;

// Public API
pub use error::InlineAncestryError;
pub use flags::NodeFlags;

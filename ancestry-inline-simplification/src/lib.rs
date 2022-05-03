pub use ancestry_common::{LargeSignedInteger, SignedInteger};

mod ancestry_overlapper;
mod error;
mod flags;
mod node_heap;
mod segments;
mod update_ancestry;
mod util;

pub(crate) use ancestry_overlapper::AncestryOverlapper;
pub(crate) use segments::*;

pub mod node;
pub mod population;

// Public API
pub use error::InlineAncestryError;
pub use flags::NodeFlags;
pub use population::Population;

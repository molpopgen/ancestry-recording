pub use ancestry_common::{LargeSignedInteger, NodeFlags, SignedInteger};

mod ancestry;
mod simplify;

pub use ancestry::{Ancestry, AncestryRecord, Segment};
pub use simplify::simplify;

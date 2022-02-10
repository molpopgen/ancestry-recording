pub use ancestry_common::{
    LargeSignedInteger, NodeFlags, NodeId, NodeTable, Position, SignedInteger,
};

mod ancestry;
mod simplify;

pub use ancestry::{Ancestry, AncestryRecord, Descendant};
pub use simplify::simplify;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_make_node() {
        let n = NodeId { value: 0 };
        assert_eq!(n.value, 0);
    }
}

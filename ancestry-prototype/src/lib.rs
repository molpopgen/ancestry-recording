pub use ancestry_common::{NodeFlags, NodeId, NodeTable, Position, SignedInteger};

mod ancestry;

pub use ancestry::{Ancestry, AncestryRecord, Descendant};

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_make_node() {
        let n = NodeId { 0: 0 };
        assert_eq!(n.0, 0);
    }
}

use crate::{Ancestry, NodeId, NodeTable};

/// No error handling, all panics right now.
pub fn simplify(
    new_births: &[NodeId],
    ancestry: &mut Ancestry,
    nodes: &mut NodeTable,
) -> Vec<NodeId> {
    assert!(new_births.len() > 1);
    // samples must be ordered by birth time, past to present
    let sorted = new_births
        .windows(2)
        .all(|w| nodes.time(w[0]) <= nodes.time(w[1]));
    assert!(sorted);

    let mut idmap = vec![NodeId::new_null(); nodes.len()];
    let mut new_node_table = NodeTable::default();

    // now, go through the sample nodes from present to past
    for node in new_births.iter().rev() {
        assert!(nodes.flags(*node).is_sample());
        // check that ancestry[i] exists
        if let Some(record) = ancestry.get_mut(*node) {
            // check the validity of the fields
            // of ancestry[i]

            // Find overlaps with the ancestor,
            // doing the magic to prune extinct segments
        } else {
            panic!("Node {:?} is not present in the ancestry", node);
        }
    }

    std::mem::swap(nodes, &mut new_node_table);
    idmap
}

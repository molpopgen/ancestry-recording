use crate::{Ancestry, NodeId, NodeTable};

/// No error handling, all panics right now.
pub fn simplify(
    extant_samples: &[NodeId],
    ancestry: &mut Ancestry,
    nodes: &mut NodeTable,
) -> Vec<NodeId> {
    // samples must be ordered by birth time, past to present
    let sorted = extant_samples
        .windows(2)
        .all(|w| nodes.row(w[0]).time() <= nodes.row(w[1]).time());
    assert!(sorted);

    let mut idmap = vec![NodeId::new_null(); nodes.len()];
    let mut new_node_table = NodeTable::default();

    // now, go through the sample nodes from present to past
    for node in extant_samples.iter().rev() {
        // check that ancestry[i] exists
        assert!(ancestry.ancestry.contains_key(node));

        // check the validity of the fields
        // of ancestry[i]

        // Find overlaps with the ancestor,
        // doing the magic to prune extinct segments
    }

    std::mem::swap(nodes, &mut new_node_table);
    idmap
}

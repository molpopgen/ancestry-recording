use crate::{Ancestry, NodeId, NodeTable};

/// No error handling, all panics right now.
pub fn simplify(
    extant_samples: &[NodeId],
    ancestry: &mut Ancestry,
    nodes: &mut NodeTable,
) -> Vec<NodeId> {
    assert!(extant_samples.len() > 1);
    // samples must be ordered by birth time, past to present
    let sorted = extant_samples
        .windows(2)
        .all(|w| nodes.time(w[0]) <= nodes.time(w[1]));
    assert!(sorted);

    let mut idmap = vec![NodeId::new_null(); nodes.len()];
    let mut new_node_table = NodeTable::default();

    // now, go through the sample nodes from present to past
    for node in extant_samples.iter().rev() {
        assert!(nodes.flags(*node).is_sample());
        // check that ancestry[i] exists
        if let Some(record) = ancestry.ancestry.get_mut(&node) {
            // check the validity of the fields
            // of ancestry[i]
            assert!(record.ancestors.len() > 0);

            // Find overlaps with the ancestor,
            // doing the magic to prune extinct segments
        } else {
            panic!("Node {:?} is not present in the ancestry", node);
        }
    }

    std::mem::swap(nodes, &mut new_node_table);
    idmap
}

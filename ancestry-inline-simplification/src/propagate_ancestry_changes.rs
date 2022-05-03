use crate::node::Node;
use crate::node_heap::NodeHeap;
use crate::InlineAncestryError;

pub fn propagate_ancestry_changes(
    genome_length: crate::LargeSignedInteger,
    node_heap: &mut NodeHeap,
) -> Result<(), InlineAncestryError> {
    while let Some(mut n) = node_heap.pop() {
        n.preprocess(genome_length);
        let mut node = Node::from(n);
        let changed = node.update_ancestry()?;
        if changed || node.is_alive() {
            for parent in node.borrow_mut().parents.iter() {
                node_heap.push_parent(parent.clone());
            }
        }
    }
    assert!(node_heap.is_empty());
    Ok(())
}

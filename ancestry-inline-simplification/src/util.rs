use crate::HalfOpenInterval;
use crate::InlineAncestryError;
use crate::Node;
use hashbrown::HashSet;

pub(crate) fn non_overlapping_segments<T: HalfOpenInterval>(
    segments: &[T],
) -> Result<(), InlineAncestryError> {
    let not_sorted = segments.windows(2).any(|w| w[0].right() > w[1].left());

    if not_sorted {
        Err(InlineAncestryError::IntervalsError)
    } else {
        Ok(())
    }
}

pub fn all_reachable_nodes(nodes: &[Node]) -> HashSet<Node> {
    let mut reachable = HashSet::new();

    for node in nodes {
        let mut stack = vec![node.clone()];
        while let Some(popped) = stack.pop() {
            reachable.insert(popped.clone());
            for parent in &popped.borrow().parents {
                if !reachable.contains(parent) {
                    stack.push(parent.clone());
                }
            }
        }
    }

    reachable
}

pub fn validate_graph(
    nodes: &[Node],
    genome_length: crate::LargeSignedInteger,
) -> Result<(), InlineAncestryError> {
    let reachable = all_reachable_nodes(nodes);

    for node in &reachable {
        node.non_overlapping_segments()?;
        let borrowed_node = node.borrow();
        if borrowed_node.is_alive() {
            assert_eq!(borrowed_node.ancestry.len(), 1);
            assert!(borrowed_node
                .ancestry
                .iter()
                .all(|a| a.left() == 0 && a.right() == genome_length));
        }
        for child in borrowed_node.children.keys() {
            if !reachable.contains(child) {
                return Err(InlineAncestryError::UnreachableChild);
            }
        }
    }

    Ok(())
}

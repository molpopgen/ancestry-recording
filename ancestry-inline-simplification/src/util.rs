use crate::HalfOpenInterval;
use crate::InlineAncestryError;
use crate::Node;
use hashbrown::HashSet;

pub(crate) fn non_overlapping_segments<T: HalfOpenInterval>(
    segments: &[T],
) -> Result<(), InlineAncestryError> {
    let sorted = segments.windows(2).all(|w| w[0].left() < w[1].left());

    if sorted {
        Ok(())
    } else {
        Err(InlineAncestryError::IntervalsError)
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

pub fn validate_graph(nodes: &[Node]) -> Result<(), InlineAncestryError> {
    let reachable = all_reachable_nodes(nodes);

    for node in &reachable {
        for child in node.borrow().children.keys() {
            if !reachable.contains(child) {
                return Err(InlineAncestryError::UnreachableChild);
            }
        }
    }

    Ok(())
}

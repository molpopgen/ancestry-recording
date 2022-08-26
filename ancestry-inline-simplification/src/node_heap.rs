use crate::node::Node;
use crate::InlineAncestryError;
use hashbrown::HashSet;
use std::collections::BinaryHeap;

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone, Copy)]
enum NodeType {
    Parent,
    Birth,
    Death,
}

#[derive(Debug)]
pub(crate) struct PrioritizedNode {
    node: Node,
    node_type: NodeType,
}

impl PartialEq for PrioritizedNode {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.node.as_ptr(), other.node.as_ptr())
    }
}

impl PartialOrd for PrioritizedNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            (self.node.borrow().birth_time, self.node_type)
                .cmp(&(other.node.borrow().birth_time, other.node_type)),
        )
    }
}

impl Eq for PrioritizedNode {}

impl Ord for PrioritizedNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PrioritizedNode {
    fn new(node: Node, node_type: NodeType) -> Self {
        Self { node, node_type }
    }

    fn get(self) -> Node {
        self.node
    }

    fn is_death(&self) -> bool {
        matches!(self.node_type, NodeType::Death)
    }

    pub fn preprocess(&mut self, genome_length: crate::LargeSignedInteger) {
        if self.is_death() {
            self.node.borrow_mut().kill(genome_length);
            debug_assert!(!self.node.is_alive());
            debug_assert!(self.node.borrow().ancestry.is_empty());
        }
    }
}

impl From<PrioritizedNode> for Node {
    fn from(value: PrioritizedNode) -> Self {
        value.get()
    }
}

pub struct NodeHeap {
    heap: BinaryHeap<PrioritizedNode>,
    in_heap: HashSet<Node>,
}

impl NodeHeap {
    fn push(&mut self, node: Node, node_type: NodeType) -> bool {
        if !self.in_heap.contains(&node) {
            self.in_heap.insert(node.clone());
            self.heap.push(PrioritizedNode::new(node, node_type));
            true
        } else {
            false
        }
    }

    pub(crate) fn push_parent(&mut self, node: Node) {
        let _ = self.push(node, NodeType::Parent);
    }

    pub(crate) fn pop(&mut self) -> Option<PrioritizedNode> {
        match self.heap.pop() {
            Some(x) => {
                self.in_heap.remove(&x.node);
                Some(x)
            }
            None => None,
        }
    }

    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            in_heap: HashSet::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        assert_eq!(self.heap.is_empty(), self.in_heap.is_empty());
        self.heap.is_empty()
    }

    pub fn len(&self) -> usize {
        assert_eq!(self.heap.is_empty(), self.in_heap.is_empty());
        self.heap.len()
    }

    pub fn push_birth(&mut self, node: Node) -> Result<(), InlineAncestryError> {
        if !node.is_alive() {
            Err(InlineAncestryError::DeadNode)
        } else {
            let _ = self.push(node, NodeType::Birth);
            Ok(())
        }
    }

    pub fn push_death(&mut self, node: Node) -> Result<(), InlineAncestryError> {
        if !node.is_alive() {
            Err(InlineAncestryError::DeadNode)
        } else {
            let _ = self.push(node, NodeType::Death);
            Ok(())
        }
    }

    pub fn clear(&mut self) {
        self.heap.clear();
        self.in_heap.clear();
    }
}

impl Default for NodeHeap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_heap() {
        let a = Node::new_alive(0, 1);
        let b = Node::new_alive(0, 2);

        let mut heap = NodeHeap::default();
        heap.push_birth(a.clone()).unwrap();
        heap.push_death(b).unwrap();

        // WARNING: this is a test of internal details!
        let mut birth_times = vec![];
        while let Some(x) = heap.pop() {
            birth_times.push(x.node.borrow().birth_time);
        }
        assert_eq!(birth_times, vec![2, 1]);
        assert!(heap.is_empty());
    }

    // WARNING: this is a test of internal details!
    #[test]
    fn test_node_type_ordering() {
        let p = NodeType::Parent;
        let b = NodeType::Birth;
        let d = NodeType::Death;

        assert_eq!(p, p);
        assert_eq!(d, d);
        assert_eq!(b, b);
        assert!(p < b);
        assert!(p < d);
        assert!(b < d);
    }
}

use crate::node_heap::NodeHeap;
use crate::InlineAncestryError;
use crate::{AncestrySegment, LargeSignedInteger, NodeFlags, Segment, SignedInteger};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::{cell::RefCell, ops::Deref};

// Use this over std::collections b/c the hashing
// fn is much faster. (We aren't doing cryptography.)
// TODO: See the O'Reilly book for which crate
// they recommend here.
use hashbrown::{HashMap, HashSet};

/// An `Node` is a pointer to [NodeData](NodeData).
///
/// Derefs to Rc<RefCell<Node>>, giving interior mutability.
/// Required so that we can hash Rc instances.
/// Hashing and equality are implemented with respect to the
/// underlying *pointers* and not the *data*.
#[derive(Clone)]
pub struct Node(Rc<RefCell<NodeData>>);

pub type ChildMap = HashMap<Node, Vec<Segment>>;
pub type ParentSet = HashSet<Node>;

#[derive(Clone)] // NOTE: this does not have to be Clone b/c we work via pointers
pub struct NodeData {
    pub index: SignedInteger, // TODO: remove this, as it is really only useful for debugging
    pub birth_time: LargeSignedInteger,
    pub flags: NodeFlags,
    pub parents: ParentSet,
    pub ancestry: Vec<AncestrySegment>,
    pub children: ChildMap,
}

impl Deref for Node {
    type Target = Rc<RefCell<NodeData>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.as_ptr(), other.as_ptr())
    }
}

impl Eq for Node {}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ptr().hash(state);
    }
}

impl Node {
    pub fn new_alive(index: SignedInteger, birth_time: LargeSignedInteger) -> Self {
        Self(Rc::new(RefCell::<NodeData>::new(NodeData::new_alive(
            index, birth_time,
        ))))
    }

    /// # Panics
    ///
    /// If `genome_length` < 1
    pub fn new_alive_with_ancestry_mapping_to_self(
        index: SignedInteger,
        birth_time: LargeSignedInteger,
        genome_length: LargeSignedInteger,
    ) -> Self {
        let rv = Self::new_alive(index, birth_time);
        rv.borrow_mut()
            .ancestry
            .push(AncestrySegment::new(0, genome_length, rv.clone()));
        rv
    }

    pub fn is_alive(&self) -> bool {
        self.borrow().flags.is_alive()
    }

    // FIXME: this is not a great fn to error from.
    // We should instead be checking that the right thing
    // happens at birth and then, during simplification,
    // rely on assert to find unexpected errors.
    pub fn add_parent(&mut self, parent: Node) -> Result<(), InlineAncestryError> {
        let mut sb = self.borrow_mut();
        if sb.birth_time > parent.borrow().birth_time {
            sb.parents.insert(parent);
            Ok(())
        } else {
            Err(InlineAncestryError::InvalidBirthTimeOrder {
                parent: parent.borrow().birth_time,
                child: sb.birth_time,
            })
        }
    }

    pub fn add_child_segment(
        &mut self,
        left: LargeSignedInteger,
        right: LargeSignedInteger,
        child: Node,
    ) -> Result<(), InlineAncestryError> {
        assert!(child.borrow().birth_time > self.borrow().birth_time);
        let mut b = self.borrow_mut();
        let interval = Segment::new(left, right)?;
        if let Some(v) = b.children.get_mut(&child) {
            v.push(interval);
        } else {
            b.children.insert(child, vec![interval]);
        }
        Ok(())
    }

    pub fn propagate_upwards(&mut self) -> Result<(), InlineAncestryError> {
        let mut heap = NodeHeap::new();
        heap.push(self.clone());
        while let Some(mut node) = heap.pop() {
            let changed = node.update_ancestry()?;
            node.non_overlapping_segments()?;
            // TODO: there is another flag needed here --
            // we don't need to do this for all alive nodes.
            if changed || node.is_alive() {
                for parent in node.borrow().parents.iter() {
                    heap.push(parent.clone());
                }
            }
        }
        Ok(())
    }

    #[inline(never)]
    // TODO: to dig more into the performance issues,
    // we need to move this into a separate module and
    // break it up into multiple functions, each of which is not inlined.
    fn update_ancestry(&mut self) -> Result<bool, InlineAncestryError> {
        let rv = crate::update_ancestry::update_ancestry(self);
        Ok(rv)
    }

    fn non_overlapping_segments(&self) -> Result<(), InlineAncestryError> {
        let b = self.borrow();
        crate::util::non_overlapping_segments(&b.ancestry)?;
        for (_child, segments) in b.children.iter() {
            crate::util::non_overlapping_segments(&segments)?;
        }
        Ok(())
    }
}

impl NodeData {
    pub fn new_alive(index: SignedInteger, birth_time: LargeSignedInteger) -> Self {
        Self {
            index,
            birth_time,
            flags: NodeFlags::new_alive(),
            parents: ParentSet::default(),
            ancestry: vec![],
            children: ChildMap::default(),
        }
    }
}

// This module is for experimenting with the Rc/RefCell pattern.
#[cfg(test)]
mod practice_tests {
    use super::*;

    fn remove_parent(parent: Node, child: Node) {
        child.borrow_mut().parents.remove(&parent);
    }

    // Better -- does not increase ref counts just for fn call.
    fn remove_parent_via_ref(parent: &Node, child: &Node) {
        child.borrow_mut().parents.remove(&parent);
    }

    #[test]
    fn test_interior_mutability() {
        let mut pop: Vec<Node> = vec![];

        pop.push(Node::new_alive(0, 0));
        pop.push(Node::new_alive(1, 1));

        {
            let c = pop[1].clone();
            pop[0].add_child_segment(0, 1, c).unwrap();
        }

        {
            let p = pop[0].clone();
            pop[1].add_parent(p).unwrap();
        }
        assert_eq!(Rc::strong_count(&pop[0]), 2);
        assert_eq!(Rc::strong_count(&pop[1]), 2);

        remove_parent(pop[0].clone(), pop[1].clone());
        assert_eq!(Rc::strong_count(&pop[0]), 1);
        assert_eq!(Rc::strong_count(&pop[1]), 2);
    }

    #[test]
    fn test_interior_mutability_via_ref() {
        let mut pop: Vec<Node> = vec![];

        pop.push(Node::new_alive(0, 0));
        pop.push(Node::new_alive(1, 1));

        {
            let c = pop[1].clone();
            pop[0].add_child_segment(0, 1, c).unwrap();
        }

        {
            let p = pop[0].clone();
            pop[1].add_parent(p).unwrap();
        }
        assert_eq!(Rc::strong_count(&pop[0]), 2);
        assert_eq!(Rc::strong_count(&pop[1]), 2);

        remove_parent_via_ref(&pop[0], &pop[1]);
        assert_eq!(Rc::strong_count(&pop[0]), 1);
        assert_eq!(Rc::strong_count(&pop[1]), 2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alive_node_has_ancestry_to_self() {
        let node = Node::new_alive_with_ancestry_mapping_to_self(0, 0, 10);
        assert_eq!(node.borrow().ancestry.len(), 1);
        assert!(node.borrow().ancestry[0].child == node);
    }

    #[test]
    fn test_equality() {
        let node = Node::new_alive(0, 1);
        let clone = node.clone();
        assert!(node == clone);
    }

    #[test]
    fn test_equality_after_interior_mutation() {
        let node = Node::new_alive(0, 1);
        let clone = node.clone();

        assert!(node.borrow().ancestry.is_empty());

        let another_node = Node::new_alive(0, 1);
        node.borrow_mut()
            .ancestry
            .push(AncestrySegment::new(0, 1, another_node.clone()));
        assert!(!node.borrow().ancestry.is_empty());
        assert!(!clone.borrow().ancestry.is_empty());
        assert!(node == clone);

        assert!(node.is_alive());
        node.borrow_mut().flags.remove(NodeFlags::IS_ALIVE);
        assert!(!node.is_alive());
        assert!(!clone.is_alive());
    }

    #[test]
    fn test_inequality() {
        let node = Node::new_alive(0, 1);
        let another_node = Node::new_alive(0, 1);
        assert!(node != another_node);
    }

    #[test]
    fn test_hashing() {
        let mut hash = hashbrown::HashSet::new();
        let node = Node::new_alive(0, 1);
        let clone = node.clone();
        let another_node = Node::new_alive(0, 1);

        hash.insert(node.clone());
        assert!(hash.contains(&node));
        assert!(hash.contains(&clone));
        assert!(!hash.contains(&another_node));

        node.borrow_mut()
            .ancestry
            .push(AncestrySegment::new(0, 1, another_node.clone()));
        assert!(hash.contains(&node));
        assert!(hash.contains(&clone));
        assert!(!hash.contains(&another_node));
    }
}

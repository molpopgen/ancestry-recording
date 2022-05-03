use crate::ancestry_overlapper::AncestryOverlapper;
use crate::flags::NodeFlags;
use crate::node::{Node, NodeData};
use crate::segments::HalfOpenInterval;
use crate::segments::Segment;
use crate::segments::{AncestryIntersection, AncestrySegment};
use crate::LargeSignedInteger;

// What follows is an attempt at cleaner code.
// Steps are broken into functions for testability
// of internal steps as needed.
// Functions are not inlined to aid profiling (for now).

#[inline(never)]
fn intersecting_ancestry(node: &Node) -> Vec<AncestryIntersection> {
    let mut rv = vec![];
    let node_data = node.borrow();

    for (child, segs) in &node_data.children {
        assert!(!segs.is_empty());
        for seg in segs.iter() {
            for x in child.borrow().ancestry.iter() {
                if x.overlaps(seg) {
                    rv.push(AncestryIntersection::new(
                        std::cmp::max(x.left(), seg.left()),
                        std::cmp::min(x.right(), seg.right()),
                        child.clone(),
                        x.child.clone(),
                    ));
                }
            }
        }
    }

    rv
}

#[inline(never)]
fn make_overlapper(node: &Node) -> AncestryOverlapper {
    let intersection = intersecting_ancestry(node);
    AncestryOverlapper::new(intersection)
}

#[inline(never)]
fn update_child_segments(
    node_data: &mut NodeData,
    child: &Node,
    left: LargeSignedInteger,
    right: LargeSignedInteger,
) {
    match node_data.children.get_mut(child) {
        Some(segs) => {
            let need_push = match segs.last_mut() {
                Some(seg) => {
                    if seg.right == left {
                        seg.right = right; // Squash child segs as we go.
                        false
                    } else {
                        true
                    }
                }
                None => true,
            };
            if need_push {
                let seg = Segment::new(left, right).unwrap();
                segs.push(seg);
            }
        }
        None => {
            let seg = Segment::new(left, right).unwrap();
            node_data.children.insert(child.clone(), vec![seg]);
        }
    }
}

#[inline(never)]
fn process_unary_overlap(
    left: LargeSignedInteger,
    right: LargeSignedInteger,
    overlap: &AncestryIntersection,
    node: &Node,
) -> Node {
    let mut mapped_node = overlap.ancestry_segment.child.clone();
    let in_parents = mapped_node.borrow().parents.contains(node);
    let mut node_data = node.borrow_mut();
    if in_parents && node_data.flags.contains(NodeFlags::IS_ALIVE) {
        if overlap.ancestry_segment.child.is_alive() {
            update_child_segments(&mut node_data, &overlap.mapped_node, left, right);
        } else {
            mapped_node = overlap.mapped_node.clone();
            mapped_node.borrow_mut().parents.insert(node.clone());
            update_child_segments(&mut node_data, &mapped_node, left, right);
        }
    }
    mapped_node
}

#[inline(never)]
fn get_mapped_node(
    left: LargeSignedInteger,
    right: LargeSignedInteger,
    overlaps: &[AncestryIntersection],
    node: &mut Node,
) -> Node {
    if overlaps.len() == 1 {
        process_unary_overlap(left, right, &overlaps[0], node)
    } else {
        // coalescence
        for overlap in overlaps.iter() {
            update_child_segments(&mut node.borrow_mut(), &overlap.mapped_node, left, right);
        }
        node.clone()
    }
}

#[inline(never)]
fn process_overlaps(
    overlapper: &mut AncestryOverlapper,
    input_ancestry_len: usize,
    output_ancestry_index: &mut usize,
    ancestry_change_detected: &mut bool,
    node: &mut Node,
) {
    for (left, right, overlaps) in overlapper {
        let borrowed_overlaps = overlaps.borrow();
        if !node.is_alive() {
            let mapped_node = get_mapped_node(left, right, &borrowed_overlaps, node);
            let mut borrowed_node = node.borrow_mut();
            if *output_ancestry_index < input_ancestry_len {
                // SAFETY: we checked bounds in the if statement
                let input_ancestry_seg = unsafe {
                    borrowed_node
                        .ancestry
                        .get_unchecked_mut(*output_ancestry_index)
                };
                if input_ancestry_seg.left() != left
                    || input_ancestry_seg.right() != right
                    || input_ancestry_seg.child != mapped_node
                {
                    input_ancestry_seg.segment.left = left;
                    input_ancestry_seg.segment.right = right;
                    input_ancestry_seg.child = mapped_node;
                    *output_ancestry_index += 1;
                    *ancestry_change_detected = true;
                }
            } else {
                borrowed_node
                    .ancestry
                    .push(AncestrySegment::new(left, right, mapped_node));
                *ancestry_change_detected = true;
            }
        } else {
            get_mapped_node(left, right, &borrowed_overlaps, node);
        }
    }
}

#[inline(never)]
pub(crate) fn update_ancestry(node: &mut Node) -> bool {
    let self_alive = node.is_alive();

    let mut overlapper = make_overlapper(node);
    let input_ancestry_len: usize;
    let mut output_ancestry_index: usize = 0;
    let mut ancestry_change_detected = false;

    {
        let mut borrowed_node = node.borrow_mut();
        input_ancestry_len = borrowed_node.ancestry.len();

        for child in borrowed_node.children.keys() {
            let mut mut_borrowed_child = child.borrow_mut();
            mut_borrowed_child.parents.remove(node);
        }

        borrowed_node.children.clear();
    }

    process_overlaps(
        &mut overlapper,
        input_ancestry_len,
        &mut output_ancestry_index,
        &mut ancestry_change_detected,
        node,
    );

    if !self_alive {
        // Remove trailing input ancestry
        if output_ancestry_index < input_ancestry_len {
            node.borrow_mut().ancestry.truncate(output_ancestry_index);
            ancestry_change_detected = true;
        }
    }

    for child in node.borrow_mut().children.keys() {
        child.borrow_mut().parents.insert(node.clone());
        assert!(child.borrow().parents.contains(node));
    }

    ancestry_change_detected || node.borrow().ancestry.is_empty()
}

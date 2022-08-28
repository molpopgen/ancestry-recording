use std::process::Output;

use crate::ancestry_overlapper::AncestryOverlapper;
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
fn process_overlaps(
    overlapper: &mut AncestryOverlapper,
    output_ancestry: &mut Vec<AncestrySegment>,
    node: &mut Node,
) {
    let mut borrowed_node = node.borrow_mut();
    for (left, right, overlaps) in overlapper {
        assert!(left < right);
        let mut mapped_node: Node = node.clone();
        let borrowed_overlaps = overlaps.borrow();

        if borrowed_overlaps.len() == 1 {
            mapped_node = borrowed_overlaps[0].mapped_node.clone();
            if borrowed_node.is_alive() {
                update_child_segments(&mut borrowed_node, &mapped_node, left, right);
            }
        } else {
            debug_assert!(*node == mapped_node);
            for overlap in borrowed_overlaps.iter() {
                update_child_segments(&mut borrowed_node, &overlap.mapped_node, left, right);
            }
        }
        if !borrowed_node.is_alive() {
            let need_push = match output_ancestry.last_mut() {
                Some(seg) => {
                    if seg.right() == left && seg.child == mapped_node {
                        seg.segment.right = right;
                        false
                    } else {
                        true
                    }
                }
                None => true,
            };
            if need_push {
                output_ancestry.push(AncestrySegment::new(left, right, mapped_node));
            }
        }
    }
}

#[inline(never)]
pub(crate) fn update_ancestry(node: &mut Node) -> bool {
    let self_alive = node.is_alive();

    let mut overlapper = make_overlapper(node);

    let mut output_ancestry = vec![];

    {
        let mut borrowed_node = node.borrow_mut();

        for child in borrowed_node.children.keys() {
            let mut mut_borrowed_child = child.borrow_mut();
            assert!(mut_borrowed_child.parents.contains(node));
            mut_borrowed_child.parents.remove(node);
        }

        borrowed_node.children.clear();
    }

    process_overlaps(&mut overlapper, &mut output_ancestry, node);

    //if !self_alive {
    //    // Remove trailing input ancestry
    //    if output_ancestry_index < input_ancestry_len {
    //        node.borrow_mut().ancestry.truncate(output_ancestry_index);
    //        ancestry_change_detected = true;
    //    }
    //}

    debug_assert!(!node.borrow().parents.contains(node));

    for child in node.borrow_mut().children.keys() {
        child.borrow_mut().parents.insert(node.clone());
        assert!(child.borrow().parents.contains(node));
    }

    // println!("before logic {:?} -> {:?}", output_ancestry, node.borrow().ancestry);

    let ancestry_change_detected = {
        if self_alive {
            false
        } else {
            let a = &mut node.borrow_mut().ancestry;
            std::mem::swap(a, &mut output_ancestry);
            if *a != output_ancestry {
                true
            } else {
                false
            }
        }
    };

    //if ancestry_change_detected {
    //    println!("{:?} -> {:?}", output_ancestry, node.borrow().ancestry);
    //}

    // NOTE:
    // The check on empty ancestry is challenging.
    // I *think* that this is required for tests to pass due to deaths,
    // in which case we need extra logic here to avoid doing extra work.
    ancestry_change_detected || node.borrow().ancestry.is_empty()
}

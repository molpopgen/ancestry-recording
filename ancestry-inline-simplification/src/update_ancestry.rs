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
    let mut rv2 = vec![];
    let node_data = node.borrow();

    for (child, segs) in &node_data.children {
        assert!(!segs.is_empty());

        debug_assert!(segs.windows(2).all(|w| w[0].left() < w[1].left()));

        let child_ancestry = &child.borrow().ancestry;
        debug_assert!(child_ancestry.windows(2).all(|w| w[0].left() < w[1].left()));
        debug_assert!(!child_ancestry.windows(2).any(|w| w[0].overlaps(&w[1])));

        //child_ancestry
        //    .iter()
        //    .flat_map(|s| {
        //        segs.iter().filter_map(|b| {
        //            if s.overlaps(b) {
        //                Some((s.clone(), b.clone()))
        //            } else {
        //                None
        //            }
        //        })
        //    })
        //    .map(|(a, b)| {
        //        AncestryIntersection::new(
        //            std::cmp::max(a.left(), b.left()),
        //            std::cmp::min(a.right(), b.right()),
        //            child.clone(),
        //            a.child.clone(),
        //        )
        //    })
        //    .for_each(|a| rv.push(a));

        let mut aindex = 0;
        let mut cindex = 0;

        while aindex < child_ancestry.len() && cindex < segs.len() {
            let i = &child_ancestry[aindex];
            println!("i = {:?}", i);
            let mut count = 0;
            for j in &segs[cindex..] {
                println!("j = {:?}", j);
                if i.overlaps(j) {
                    rv.push(AncestryIntersection::new(
                        std::cmp::max(i.left(), j.left()),
                        std::cmp::min(i.right(), j.right()),
                        child.clone(),
                        i.child.clone(),
                    ));
                    count += 1;
                } else {
                    break;
                }
            }
            cindex += count;
            if count == 0 {
                aindex += 1;
            }
        }

        for seg in segs.iter() {
            debug_assert!(child
                .borrow()
                .ancestry
                .windows(2)
                .all(|w| w[0].left() < w[1].left()));
            for x in child_ancestry {
                if x.overlaps(seg) {
                    rv2.push(AncestryIntersection::new(
                        std::cmp::max(x.left(), seg.left()),
                        std::cmp::min(x.right(), seg.right()),
                        child.clone(),
                        x.child.clone(),
                    ));
                }
            }
        }
    }
    assert_eq!(rv, rv2);

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

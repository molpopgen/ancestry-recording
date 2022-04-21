use crate::ancestry_overlapper::AncestryOverlapper;
use crate::flags::NodeFlags;
use crate::individual::{Individual, IndividualData};
use crate::segments::HalfOpenInterval;
use crate::segments::Segment;
use crate::segments::{AncestryIntersection, AncestrySegment};
use crate::LargeSignedInteger;

// What follows is an attempt at cleaner code.
// Steps are broken into functions for testability
// of internal steps as needed.
// Functions are not inlined to aid profiling (for now).

#[inline(never)]
fn intersecting_ancestry(individual: &Individual) -> Vec<AncestryIntersection> {
    let mut rv = vec![];
    let individual_data = individual.borrow();

    for (child, segs) in &individual_data.children {
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
fn make_overlapper(individual: &Individual) -> AncestryOverlapper {
    let intersection = intersecting_ancestry(individual);
    AncestryOverlapper::new(intersection)
}

#[inline(never)]
fn update_child_segments(
    ind: &mut IndividualData,
    child: &Individual,
    left: LargeSignedInteger,
    right: LargeSignedInteger,
) {
    match ind.children.get_mut(child) {
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
            ind.children.insert(child.clone(), vec![seg]);
        }
    }
}

#[inline(never)]
fn process_unary_overlap(
    left: LargeSignedInteger,
    right: LargeSignedInteger,
    overlap: &AncestryIntersection,
    individual: &Individual,
) -> Individual {
    let mut mapped_ind = overlap.ancestry_segment.child.clone();
    let in_parents = mapped_ind.borrow().parents.contains(individual);
    let mut individual_data = individual.borrow_mut();
    if in_parents && individual_data.flags.contains(NodeFlags::IS_ALIVE) {
        if overlap.ancestry_segment.child.is_alive() {
            update_child_segments(
                &mut individual_data,
                &overlap.mapped_individual,
                left,
                right,
            );
        } else {
            mapped_ind = overlap.mapped_individual.clone();
            mapped_ind.borrow_mut().parents.insert(individual.clone());
            update_child_segments(&mut individual_data, &mapped_ind, left, right);
        }
    }
    mapped_ind
}

#[inline(never)]
fn get_mapped_ind(
    left: LargeSignedInteger,
    right: LargeSignedInteger,
    overlaps: &[AncestryIntersection],
    individual: &mut Individual,
) -> Individual {
    if overlaps.len() == 1 {
        process_unary_overlap(left, right, &overlaps[0], individual)
    } else {
        // coalescence
        for overlap in overlaps.iter() {
            update_child_segments(
                &mut individual.borrow_mut(),
                &overlap.mapped_individual,
                left,
                right,
            );
        }
        individual.clone()
    }
}

#[inline(never)]
fn process_overlaps(
    overlapper: &mut AncestryOverlapper,
    input_ancestry_len: usize,
    output_ancestry_index: &mut usize,
    ancestry_change_detected: &mut bool,
    individual: &mut Individual,
) {
    for (left, right, overlaps) in overlapper {
        let borrowed_overlaps = overlaps.borrow();
        if !individual.is_alive() {
            let mapped_ind = get_mapped_ind(left, right, &borrowed_overlaps, individual);
            let mut borrowed_ind = individual.borrow_mut();
            if *output_ancestry_index < input_ancestry_len {
                // SAFETY: we checked bounds in the if statement
                let input_ancestry_seg = unsafe {
                    borrowed_ind
                        .ancestry
                        .get_unchecked_mut(*output_ancestry_index)
                };
                if input_ancestry_seg.left() != left
                    || input_ancestry_seg.right() != right
                    || input_ancestry_seg.child != mapped_ind
                {
                    input_ancestry_seg.segment.left = left;
                    input_ancestry_seg.segment.right = right;
                    input_ancestry_seg.child = mapped_ind;
                    *output_ancestry_index += 1;
                    *ancestry_change_detected = true;
                }
            } else {
                borrowed_ind
                    .ancestry
                    .push(AncestrySegment::new(left, right, mapped_ind));
                *ancestry_change_detected = true;
            }
        } else {
            get_mapped_ind(left, right, &borrowed_overlaps, individual);
        }
    }
}

#[inline(never)]
pub(crate) fn update_ancestry(individual: &mut Individual) -> bool {
    let self_alive = individual.is_alive();

    let mut overlapper = make_overlapper(individual);
    let input_ancestry_len: usize;
    let mut output_ancestry_index: usize = 0;
    let mut ancestry_change_detected = false;

    {
        let mut borrowed_ind = individual.borrow_mut();
        input_ancestry_len = borrowed_ind.ancestry.len();

        for child in borrowed_ind.children.keys() {
            let mut mut_borrowed_child = child.borrow_mut();
            mut_borrowed_child.parents.remove(individual);
        }

        borrowed_ind.children.clear();
    }

    process_overlaps(
        &mut overlapper,
        input_ancestry_len,
        &mut output_ancestry_index,
        &mut ancestry_change_detected,
        individual,
    );

    if !self_alive {
        // Remove trailing input ancestry
        if output_ancestry_index < input_ancestry_len {
            individual
                .borrow_mut()
                .ancestry
                .truncate(output_ancestry_index);
            ancestry_change_detected = true;
        }
    }

    for child in individual.borrow_mut().children.keys() {
        child.borrow_mut().parents.insert(individual.clone());
        assert!(child.borrow().parents.contains(individual));
    }

    ancestry_change_detected || individual.borrow().ancestry.is_empty()
}

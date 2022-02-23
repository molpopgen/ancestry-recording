use crate::{Ancestry, AncestryRecord, NodeId, Segment, SignedInteger};

struct SimplificationInternalState {
    idmap: Vec<NodeId>,
    is_sample: Vec<bool>,
    next_output_node_id: SignedInteger,
}

struct SegmentQueue {
    segments: Vec<Segment>,
}

impl SimplificationInternalState {
    fn new(ancestry: &mut Ancestry, samples: &[NodeId]) -> Self {
        let mut is_sample = vec![false; ancestry.ancestry.len()];
        for s in samples {
            assert!(s.value >= 0);
            let u = s.value as usize;
            assert!(u < ancestry.ancestry.len());
            if is_sample[u] {
                panic!("duplicate samples");
            }
            is_sample[u] = true;
        }
        Self {
            idmap: vec![NodeId::new_null(); ancestry.ancestry.len()],
            is_sample,
            next_output_node_id: 0,
        }
    }
}

impl SegmentQueue {
    fn new_from_input_edges(input: &[Segment]) -> Self {
        let mut segments = input.to_vec();
        segments.sort_by(|a, b| match a.left.partial_cmp(&b.left) {
            Some(std::cmp::Ordering::Less) => std::cmp::Ordering::Greater,
            Some(std::cmp::Ordering::Greater) => std::cmp::Ordering::Less,
            Some(x) => x,
            None => panic!("unexpected None"),
        });

        Self { segments }
    }
}

fn process_input_record(record: &mut AncestryRecord, state: &mut SimplificationInternalState) {}

/// No error handling, all panics right now.
pub fn simplify(samples: &[NodeId], ancestry: &mut Ancestry) -> Vec<NodeId> {
    assert!(samples.len() > 1);
    // input data must be ordered by birth time, past to present
    // NOTE: this check would be more efficient if done in the
    // main iter_mut loop below.
    let sorted = ancestry
        .ancestry
        .windows(2)
        .all(|w| w[0].birth_time <= w[1].birth_time);
    if !sorted {
        panic!("input Ancestry must be sorted by birth time from past to present");
    }

    let mut state = SimplificationInternalState::new(ancestry, samples);

    for record in ancestry.ancestry.iter_mut().rev() {
        process_input_record(record, &mut state);
    }

    state.idmap
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_segments() -> Vec<Segment> {
        let mut rv = vec![];
        rv.push(Segment {
            descendant: ancestry_common::NodeId { value: 0 },
            left: ancestry_common::Position { value: 2 },
            right: ancestry_common::Position { value: 3 },
        });
        rv.push(Segment {
            descendant: ancestry_common::NodeId { value: 0 },
            left: ancestry_common::Position { value: 0 },
            right: ancestry_common::Position { value: 5 },
        });

        rv.push(Segment {
            descendant: ancestry_common::NodeId { value: 0 },
            left: ancestry_common::Position { value: 1 },
            right: ancestry_common::Position { value: 8 },
        });

        rv
    }

    #[test]
    fn test_segment_queue_creation() {
        let segments = make_segments();
        let q = SegmentQueue::new_from_input_edges(&segments);
        let sorted = q.segments.windows(2).all(|w| w[0].left >= w[1].left);
        assert!(sorted);
    }
}

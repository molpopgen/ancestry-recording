use crate::{Ancestry, AncestryRecord, LargeSignedInteger, Segment, SignedInteger};

struct SimplificationInternalState {
    idmap: Vec<SignedInteger>,
    is_sample: Vec<bool>,
    next_output_node_id: SignedInteger,
}

#[derive(Default)]
struct SegmentQueue {
    segments: Vec<Segment>,
}

impl SimplificationInternalState {
    fn new(ancestry: &mut Ancestry, samples: &[SignedInteger]) -> Self {
        let mut is_sample = vec![false; ancestry.ancestry.len()];
        for s in samples {
            assert!(*s >= 0);
            let u = *s as usize;
            assert!(u < ancestry.ancestry.len());
            if is_sample[u] {
                panic!("duplicate samples");
            }
            is_sample[u] = true;
        }
        Self {
            idmap: vec![-1; ancestry.ancestry.len()],
            is_sample,
            next_output_node_id: 0,
        }
    }
}

impl SegmentQueue {
    fn new_from_vec(input: Vec<Segment>) -> Self {
        let mut segments = input;
        segments.sort_by(|a, b| {
            std::cmp::Reverse(a.left)
                .partial_cmp(&std::cmp::Reverse(b.left))
                .unwrap()
        });
        Self { segments }
    }

    // NOTE: not clear this should be in the API...
    fn new_from_input_edges(input: &[Segment]) -> Self {
        let mut segments = input.to_vec();
        segments.sort_by(|a, b| {
            std::cmp::Reverse(a.left)
                .partial_cmp(&std::cmp::Reverse(b.left))
                .unwrap()
        });
        Self { segments }
    }

    fn clear(&mut self) {
        self.segments.clear();
    }

    fn add_segment(&mut self, segment: Segment) {
        self.segments.push(segment);
    }

    fn finalize(&mut self) {
        self.segments.sort_by(|a, b| {
            std::cmp::Reverse(a.left)
                .partial_cmp(&std::cmp::Reverse(b.left))
                .unwrap()
        });
    }

    fn pop(&mut self) -> Option<Segment> {
        self.segments.pop()
    }

    fn enqueue(&mut self, segment: Segment) {
        let mut insertion = usize::MAX;

        for (i, v) in self.segments.iter().rev().enumerate() {
            if segment.left < v.left {
                insertion = self.segments.len() - i;
                break;
            }
        }
        self.segments.insert(insertion, segment);
        let sorted = self.segments.windows(2).all(|w| w[0].left >= w[1].left);
        assert!(sorted);
    }
}

fn process_input_record(record: &mut AncestryRecord, state: &mut SimplificationInternalState) {}

/// No error handling, all panics right now.
pub fn simplify(samples: &[SignedInteger], ancestry: &mut Ancestry) -> Vec<SignedInteger> {
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
    use std::arch::x86_64::_MM_SET_FLUSH_ZERO_MODE;

    use super::*;

    fn make_segments() -> Vec<Segment> {
        let mut rv = vec![];
        rv.push(Segment {
            descendant: 0,
            left: 3,
            right: 4,
        });
        rv.push(Segment {
            descendant: 0,
            left: 1,
            right: 5,
        });

        rv.push(Segment {
            descendant: 0,
            left: 1,
            right: 8,
        });

        rv
    }

    #[test]
    fn test_segment_queue_creation() {
        let segments = make_segments();
        let q = SegmentQueue::new_from_vec(segments);
        let sorted = q.segments.windows(2).all(|w| w[0].left >= w[1].left);
        assert!(sorted);
    }

    #[test]
    fn test_segment_queue_enqueue() {
        let segments = make_segments();
        let mut q = SegmentQueue::default();
        for s in segments.into_iter() {
            q.segments.push(s);
        }
        q.finalize();
        let sorted = q.segments.windows(2).all(|w| w[0].left >= w[1].left);
        assert!(sorted);
        q.enqueue(Segment {
            descendant: 0,
            left: 2,
            right: 5,
        });
        q.enqueue(Segment {
            descendant: 0,
            left: 0,
            right: 5,
        });
    }
}

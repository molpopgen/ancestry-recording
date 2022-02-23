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

fn process_input_record(record: &mut AncestryRecord, state: &mut SimplificationInternalState) {}

/// No error handling, all panics right now.
pub fn simplify(samples: &[NodeId], ancestry: &mut Ancestry) -> Vec<NodeId> {
    assert!(samples.len() > 1);
    // input data must be ordered by birth time, past to present
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

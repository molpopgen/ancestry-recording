use crate::{Ancestry, LargeSignedInteger, NodeId, Time};

/// No error handling, all panics right now.
pub fn simplify(new_births: &[NodeId], ancestry: &mut Ancestry) -> Vec<NodeId> {
    assert!(new_births.len() > 1);
    // samples must be ordered by birth time, past to present
    let sorted = new_births.windows(2).all(|w| {
        ancestry.ancestry[w[0].value as usize].birth_time
            <= ancestry.ancestry[w[1].value as usize].birth_time
    });
    assert!(sorted);

    let mut idmap = vec![NodeId::new_null(); ancestry.ancestry.len()];

    // now, go through the sample nodes from present to past
    let mut last_time = Time {
        value: LargeSignedInteger::MAX,
    };
    for node in ancestry.ancestry.iter().rev() {
        assert!(node.birth_time <= last_time);
        last_time = node.birth_time;
    }

    idmap
}

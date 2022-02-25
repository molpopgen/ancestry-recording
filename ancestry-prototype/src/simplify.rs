use crate::{Ancestry, LargeSignedInteger, Segment, SignedInteger};
use std::collections::BinaryHeap;

struct SimplificationInternalState {
    idmap: Vec<SignedInteger>,
    is_sample: Vec<bool>,
    next_output_node_id: SignedInteger,
}

impl SimplificationInternalState {
    fn new(ancestry: &mut Ancestry, samples: &[SignedInteger]) -> Self {
        let mut is_sample = vec![false; ancestry.ancestry.len()];
        let mut idmap = vec![-1; ancestry.ancestry.len()];
        let mut next_output_node_id = 0;

        // Unlike tskit, we do 2
        // passes here so that the output ids
        // do not depend on the order specified in
        // the samples list.
        for s in samples {
            assert!(*s >= 0);
            let u = *s as usize;
            assert!(u < ancestry.ancestry.len());
            if is_sample[u] {
                panic!("duplicate samples");
            }
            is_sample[u] = true;
        }

        let mut last_birth_time = LargeSignedInteger::MAX;
        for a in ancestry.edges.iter_mut().rev() {
            // Validate input order "on demand"
            if a.birth_time > last_birth_time {
                panic!("input data must be sorted by birth time from past to present");
            }
            last_birth_time = a.birth_time;
            let u = a.node as usize;

            // Clear out pre-existing ancestry
            ancestry.ancestry[u].ancestry.clear();

            if is_sample[u] {
                // add an output id
                idmap[u] = next_output_node_id;
                next_output_node_id += 1;

                // Add initial ancestry for this node
                // FIXME: this is a problem!!
                // Sample nodes must have their "ancestry
                // mapping to self" somehow "fixed"
                // to map to the "right place".
                ancestry.ancestry[u].ancestry.push(Segment::new(
                    idmap[u],
                    0,
                    ancestry.genome_length,
                ));
            }
        }
        Self {
            idmap,
            is_sample,
            next_output_node_id,
        }
    }
}

/// No error handling, all panics right now.
/// This is the logic from the SI material of the
/// PLoS Comp Bio paper.
pub fn simplify(samples: &[SignedInteger], ancestry: &mut Ancestry) -> Vec<SignedInteger> {
    assert!(samples.len() > 1);
    assert_eq!(ancestry.edges.len(), ancestry.ancestry.len());

    // NOTE: this check is now done above, during state setup.
    // This arguably improves perf, but at the potential cost
    // of obscuring logic by mixing it with input verification.
    // input data must be ordered by birth time, past to present
    // NOTE: this check would be more efficient if done in the
    // main iter_mut loop below.
    //let sorted = ancestry
    //    .edges
    //    .windows(2)
    //    .all(|w| w[0].birth_time <= w[1].birth_time);
    //if !sorted {
    //    panic!("input Ancestry must be sorted by birth time from past to present");
    //}

    // clear existing ancestry
    // for i in ancestry.ancestry.iter_mut() {
    //     i.ancestry.clear();
    // }

    let mut state = SimplificationInternalState::new(ancestry, samples);

    let edges = &mut ancestry.edges;
    let ancestry_data = &mut ancestry.ancestry;
    let mut queue = BinaryHeap::<Segment>::default();

    for record in edges.iter_mut().rev() {
        for e in record.descendants.iter() {
            for x in ancestry_data[e.node as usize].ancestry.iter() {
                if x.right > e.left && e.right > x.left {
                    queue.push(Segment::new(
                        x.node,
                        std::cmp::max(x.left, e.left),
                        std::cmp::min(x.right, e.right),
                    ));
                }
            }
        }

        record.descendants.clear();

        let mut output_node: SignedInteger = -1;

        while !queue.is_empty() {
            let l = queue.peek().unwrap().left;
            let mut r = ancestry.genome_length;
            let mut overlaps = vec![];

            while !queue.is_empty() && queue.peek().unwrap().left == l {
                let x = queue.pop().unwrap();
                overlaps.push(x);
                r = std::cmp::min(r, x.right);
            }
            match queue.peek() {
                Some(x) => r = std::cmp::min(r, x.left),
                None => (),
            }

            assert!(!overlaps.is_empty());

            if overlaps.len() == 1 {
                let mut x = overlaps[0];
                let mut alpha = x;
                match queue.peek() {
                    Some(seg) => {
                        if seg.left < x.right {
                            alpha = Segment::new(x.node, seg.left, x.right);
                            x.left = seg.left;
                            queue.push(x);
                        }
                    }
                    None => (),
                }
                ancestry_data[record.node as usize].ancestry.push(alpha);
            } else {
                if output_node == -1 {
                    output_node = state.next_output_node_id;
                    state.next_output_node_id += 1;
                    state.idmap[record.node as usize] = output_node;
                }
                assert!(l < r);
                let alpha = Segment::new(output_node, l, r);
                for o in overlaps.iter_mut() {
                    record.descendants.push(Segment::new(o.node, l, r));
                    if o.right > r {
                        o.left = r;
                        queue.push(*o);
                    }
                }
                println!("adding ancestry for {} -> {}", record.node, alpha.node);
                ancestry_data[record.node as usize].ancestry.push(alpha);
            }
        }
    }

    // Remap node ids.

    // for i in samples.iter() {
    //     assert!( push.idmap[*i as usize] >= 0);
    //     let u = *i as usize;
    //     state.idmap[u] = (state.idmap[u] - state.next_output_node_id).abs()-1;
    //     assert!(state.idmap[u] >=0);
    // }
    for i in state.idmap.iter_mut() {
        if *i >= 0 {
            let u = *i;
            *i = (*i - state.next_output_node_id).abs() - 1;
            assert!(*i >= 0);
        }
    }

    for (i, j) in edges.iter_mut().zip(ancestry_data.iter_mut()) {
        assert!(i.node == j.node);
        i.node = state.idmap[i.node as usize];
        j.node = state.idmap[j.node as usize];

        // FIXME: this is wrong.
        // We are mapping an initial output ID to
        // another output ID assuming it the ID
        // is actually an input ID.
        // This error causes 2ndary simplifications to barf.
        for s in i.descendants.iter_mut() {
            s.node = (s.node - state.next_output_node_id).abs() - 1;
        }
        for s in j.ancestry.iter_mut() {
            s.node = (s.node - state.next_output_node_id).abs() - 1;
        }
    }

    edges.retain(|r| r.node != -1);
    ancestry_data.retain(|r| r.node != -1);

    ancestry.validate_post_simplification();

    state.idmap
}

#[cfg(test)]
mod tests {
    use std::{usize, vec};

    use super::*;

    fn make_segments() -> Vec<Segment> {
        let mut rv = vec![];
        rv.push(Segment {
            node: 0,
            left: 3,
            right: 4,
        });
        rv.push(Segment {
            node: 0,
            left: 1,
            right: 5,
        });

        rv.push(Segment {
            node: 0,
            left: 1,
            right: 8,
        });

        rv
    }

    fn feb_11_example() -> Ancestry {
        // 11 Feb example from my notebook

        // leftmost xover pos'n
        let x = 50;

        // rightmost xover pos'n
        let y = 60;

        // "Genome length"
        let l = 100;

        let mut a = Ancestry::new(l);

        let node = a.record_node(0);
        assert_eq!(node, 0);
        a.record_transmission(node, 2, 0, x);
        let node = a.record_node(0);
        assert_eq!(node, 1);
        a.record_transmission(node, 2, x, l);
        a.record_transmission(node, 3, 0, l);
        let node = a.record_node(1);
        assert_eq!(node, 2);
        a.record_transmission(node, 5, 0, y);
        let node = a.record_node(1);
        assert_eq!(node, 3);
        a.record_transmission(node, 4, 0, l);
        a.record_transmission(node, 5, y, l);
        let node = a.record_node(2);
        assert_eq!(node, 4);
        let node = a.record_node(2);
        assert_eq!(node, 5);

        a
    }

    #[test]
    fn test_simplification() {
        {
            let mut a = feb_11_example();
            let samples = vec![4, 5];
            let _idmap = simplify(&samples, &mut a);

            for (i, e) in a.edges.iter().enumerate() {
                assert_eq!(i, e.node as usize);
            }
        }
    }

    fn simplify_feb_11_with_samples(samples: &[SignedInteger]) -> (Vec<SignedInteger>, Ancestry) {
        let mut a = feb_11_example();
        let idmap = simplify(&samples, &mut a);
        (idmap, a)
    }

    #[test]
    fn test_simplification_independence_from_sample_order() {
        {
            let (idmap_1, _) = simplify_feb_11_with_samples(&vec![4, 5]);
            let (idmap_2, _) = simplify_feb_11_with_samples(&vec![5, 4]);

            for (i, j) in idmap_1.iter().zip(idmap_2.iter()) {
                assert_eq!(*i, *j);
            }
        }
    }

    #[test]
    fn test_simplify_record_simplify() {
        let samples = vec![4, 5];
        let (idmap, mut anc) = simplify_feb_11_with_samples(&samples);

        let mut parents = vec![];
        for i in samples.iter() {
            assert!(*i >= 0);
            parents.push(idmap[*i as usize]);
        }

        println!("ancestry after first simplification");
        for a in anc.ancestry.iter() {
            println!("node {}", a.node);
            for d in a.ancestry.iter() {
                println!("segs = L{}, R{}, N{}", d.left, d.right, d.node);
            }
        }
        println!("done");
        println!("edges after first simplification");
        for a in anc.edges.iter() {
            println!("node {}", a.node);
            for d in a.descendants.iter() {
                println!("segs = L{}, R{}, N{}", d.left, d.right, d.node);
            }
        }
        println!("done");

        //xover pos'ns
        let a = 25;
        let b = 75;
        let c = 10;
        let d = 90;

        let maxnode = *parents.iter().max().unwrap();
        assert_eq!(maxnode, 3);
        let mut samples = vec![];
        // Make 4 offspring nodes for the final generation
        for i in 0..4 {
            let n = anc.record_node(3);
            assert_eq!(n, maxnode + i + 1);
            samples.push(n);
        }

        println!("samples: ");
        for i in samples.iter() {
            println!("sample = {}", *i);
        }
        println!("done");

        // We want to create coalescences in both node 4 and node 5
        anc.record_transmission(parents[0], 4, a, anc.genome_length);
        anc.record_transmission(parents[0], 6, b, anc.genome_length);
        anc.record_transmission(parents[1], 5, 0, d);
        anc.record_transmission(parents[1], 7, c, d);

        let idmap = simplify(&samples, &mut anc);
        println!("idmap after 2nd:");
        for (i, j) in idmap.iter().enumerate() {
            println!("{} -> {}", i, *j);
        }
        for i in samples.iter() {
            assert!(idmap[*i as usize] >= 0);
        }
        for (i, e) in anc.edges.iter().enumerate() {
            assert_eq!(i, e.node as usize);
        }
        println!("ancestry after second simplification");
        for a in anc.ancestry.iter() {
            println!("node = {}", a.node);
            for d in a.ancestry.iter() {
                println!("segs = L{}, R{}, N{}", d.left, d.right, d.node);
            }
        }
        println!("done");
        println!("edges after second simplification");
        for a in anc.edges.iter() {
            println!("node {}", a.node);
            for d in a.descendants.iter() {
                println!("segs = L{}, R{}, N{}", d.left, d.right, d.node);
            }
        }
        println!("done");
        // the remapped parents
        for i in parents.iter_mut() {
            let u = *i;
            assert!(idmap[*i as usize] >= 0);
            *i = idmap[*i as usize];
            println!("parent {} remapped to {}:", u, *i);
            for d in anc.ancestry[*i as usize].ancestry.iter() {
                println!("segs = L{}, R{}, N{}", d.left, d.right, d.node);
            }
        }
    }
}

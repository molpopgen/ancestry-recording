use crate::indexed_node::{NodeTable, ParentSet};
use crate::InlineAncestryError;
use crate::LargeSignedInteger;
use crate::SignedInteger;
// use neutral_evolution::EvolveAncestry;
use std::collections::BinaryHeap;

struct PrioritizedNode {
    index: usize,
    birth_time: LargeSignedInteger,
    node_type: crate::node_heap::NodeType,
}

impl PartialEq for PrioritizedNode {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl PartialOrd for PrioritizedNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some((self.birth_time, self.node_type).cmp(&(other.birth_time, other.node_type)))
    }
}

pub struct NodeHeap {
    heap: BinaryHeap<PrioritizedNode>,
}

#[derive(Default)]
pub struct IndexedPopulation {
    pub nodes: NodeTable,
    pub genome_length: LargeSignedInteger,
    pub births: Vec<usize>,
    pub deaths: Vec<usize>,
    pub next_replacement: usize,
}

impl IndexedPopulation {
    pub fn new(
        popsize: SignedInteger,
        genome_length: LargeSignedInteger,
    ) -> Result<Self, InlineAncestryError> {
        if genome_length > 0 {
            let mut nodes = NodeTable::default();

            for i in 0..popsize {
                let node = nodes.new_birth(0, genome_length, ParentSet::default());
                match node {
                    Ok(v) => assert_eq!(v, i as usize),
                    Err(v) => panic!("{}", v), // Should be an error.
                }
            }

            Ok(Self {
                nodes,
                genome_length,
                births: vec![],
                deaths: vec![],
                next_replacement: 0,
            })
        } else {
            Err(InlineAncestryError::InvalidGenomeLength { l: genome_length })
        }
    }

    fn add_birth(
        &mut self,
        birth_time: LargeSignedInteger,
        parent_indexes: &[usize],
    ) -> Result<usize, usize> {
        match self.nodes.new_birth(
            birth_time,
            self.genome_length,
            ParentSet::from_iter(parent_indexes.iter().map(|v| *v)),
        ) {
            Ok(b) => {
                self.births.push(b);
                Ok(b)
            }
            Err(b) => Err(b),
        }
    }
}

#[cfg(test)]
mod test_indexed_population {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut pop = IndexedPopulation::new(2, 10).unwrap();
        let birth_time: crate::LargeSignedInteger = 1;
        let parent_0 = 0_usize;
        let parent_1 = 1_usize;
        let b = pop.add_birth(birth_time, &[parent_0, parent_1]);
        assert_eq!(b, Ok(2));

        // DESIGN NOTE:
        // Adding a birth DOES NOT increase the
        // reference count of a parent!!!!!!!!!!!!!!!!!!!!!!!!!
        // Simplification will handle that later!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!1
        // The reason is:
        // 1. We do births, update ref counts.
        // 2. We simplify, which will first (probably?) set ref counts to zero.
        // 3. During simplification, we increment the ref counts.
        // 4. Let's not do the same stuff over and over.
        assert_eq!(pop.nodes.counts[parent_0], 1);
        assert_eq!(pop.nodes.counts[parent_1], 1);
    }

    //#[test]
    //fn test_forced_recycling() {
    //    let mut pop = IndexedPopulation::new(2, 10).unwrap();
    //    let birth_time: crate::LargeSignedInteger = 1;
    //    let parent_0 = 0_usize;
    //    let parent_1 = 1_usize;
    //    // pop.queue.push(0); // FIXME: not working via public interface
    //    pop.add_birth(birth_time, &[parent_0, parent_1]).unwrap();
    //}

    #[test]
    fn test_bad_parents() {
        let mut pop = IndexedPopulation::new(2, 10).unwrap();
        let birth_time: crate::LargeSignedInteger = 1;
        let parent_0 = 2_usize;
        assert!(pop.add_birth(birth_time, &[parent_0]).is_err());
    }
}

// impl EvolveAncestry for IndexedPopulation {
//     fn genome_length(&self) -> LargeSignedInteger {
//         self.genome_length
//     }
//
//     fn setup(&mut self, _final_time: LargeSignedInteger) {}
//
//     fn generate_deaths(&mut self, death: &mut neutral_evolution::Death) -> usize {
//         self.replacements.clear();
//         self.next_replacement = 0;
//
//         // FIXME: this is wrong
//         for i in 0..self.nodes.counts.len() {
//             if death.dies() {
//                 self.replacements.push(i);
//             }
//         }
//
//         self.replacements.len()
//     }
// }

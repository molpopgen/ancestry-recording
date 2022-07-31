use crate::indexed_node::{NodeTable, ParentSet};
use crate::InlineAncestryError;
use crate::LargeSignedInteger;
use crate::SignedInteger;

#[derive(Default)]
pub struct IndexedPopulation {
    pub nodes: NodeTable,
    pub genome_length: LargeSignedInteger,
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
        self.nodes.new_birth(
            birth_time,
            self.genome_length,
            ParentSet::from_iter(parent_indexes.iter().map(|v| *v)),
        )
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
        assert_eq!(pop.nodes.counts[parent_0], 2); // DESIGN ?? -- do we want to do this at birth,
                                                   // or just when simplification happens?
                                                   // Likely the latter, else we do a bunch of
                                                   // stuff multiple times.
        assert_eq!(pop.nodes.counts[parent_1], 2);
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

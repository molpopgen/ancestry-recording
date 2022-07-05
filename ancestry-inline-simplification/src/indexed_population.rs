use crate::LargeSignedInteger;
#[derive(Default)]
pub struct IndexedPopulation {
    nodes: Vec<crate::indexed_node::Node>,
    counts: Vec<i32>,
    // FIFO queue to recycle indexes of extinct (zero) counts
    queue: Vec<usize>,
}

impl IndexedPopulation {
    fn add_node(&mut self, birth_time: LargeSignedInteger, parent_indexes: &[usize]) {
        // FIXME: Should just deal DIRECTLY with the option
        // We want to re-use stuff that is extinct.
        let index = self.get_next_node_index();

        let mut parents = crate::indexed_node::ParentSet::default();

        for parent in parent_indexes {
            //FIXME: parents must exist...
            //FIXME: increase parent count by 1
            parents.insert(*parent);
        }

        // FIXME: should be a constructor call
        let node = crate::indexed_node::Node {
            index,
            birth_time,
            flags: crate::NodeFlags::IS_ALIVE,
            parents,
            ancestry: vec![], // FIXME: must map to self
            children: crate::indexed_node::ChildMap::default(),
        };
        self.nodes.push(node);
        self.counts.push(1);
    }

    fn get_next_node_index(&mut self) -> usize {
        match self.queue.pop() {
            Some(value) => value,
            None => self.nodes.len(),
        }
    }
}

#[cfg(test)]
mod test_indexed_population {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut pop = IndexedPopulation::default();
        let birth_time: crate::LargeSignedInteger = 1;
        let parent_0 = 0_usize;
        let parent_1 = 1_usize;
        pop.add_node(birth_time, &[parent_0, parent_1]);
    }
}

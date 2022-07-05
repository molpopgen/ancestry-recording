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
        //FIXME: parents must exist...
        //FIXME: increase parent count by 1
        let mut parents = crate::indexed_node::ParentSet::default();
        for parent in parent_indexes {
            //FIXME: parents must exist...
            //FIXME: increase parent count by 1
            parents.insert(*parent);
        }
        match self.queue.pop() {
            Some(index) => {
                // FIXME: this should pass on a set!
                self.nodes[index].recycle(birth_time, parents);
                self.counts[index] += 1;
            }
            None => {
                let index = self.nodes.len();
                self.nodes.push(crate::indexed_node::Node::new_birth(
                    index, birth_time, parents,
                ));
            }
        }
        self.counts.push(1);
        debug_assert_eq!(self.nodes.len(), self.counts.len());
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

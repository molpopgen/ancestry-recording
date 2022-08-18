use crate::indexed_node::{NodeTable, ParentSet};
use crate::node_heap::NodeType;
use crate::HalfOpenInterval;
use crate::InlineAncestryError;
use crate::LargeSignedInteger;
use crate::SignedInteger;
use neutral_evolution::EvolveAncestry;
use std::collections::BinaryHeap;

#[derive(Debug)]
struct PrioritizedNode {
    index: usize,
    birth_time: LargeSignedInteger,
    node_type: NodeType,
}

impl PartialEq for PrioritizedNode {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl Eq for PrioritizedNode {}

impl PartialOrd for PrioritizedNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some((self.birth_time, self.node_type).cmp(&(other.birth_time, other.node_type)))
    }
}

impl Ord for PrioritizedNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, Default)]
pub struct NodeHeap {
    heap: BinaryHeap<PrioritizedNode>,
    in_heap: Vec<bool>,
}

impl NodeHeap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn initialize(&mut self, num_nodes: usize) {
        assert!(self.heap.is_empty());
        self.in_heap.resize(num_nodes, false);
        self.in_heap.fill(false);
    }

    fn pop(&mut self) -> Option<PrioritizedNode> {
        match self.heap.pop() {
            Some(node) => {
                self.in_heap[node.index] = false;
                Some(node)
            }
            None => None,
        }
    }

    pub fn push_if(&mut self, index: usize, birth_time: LargeSignedInteger, node_type: NodeType) {
        if !self.in_heap[index] {
            self.in_heap[index] = true;
            self.heap.push(PrioritizedNode {
                index,
                birth_time,
                node_type,
            });
        }
    }
}

#[derive(Default)]
pub struct IndexedPopulation {
    pub nodes: NodeTable,
    pub genome_length: LargeSignedInteger,
    pub alive_nodes: Vec<usize>,
    pub births: Vec<usize>,
    pub deaths: Vec<usize>,
    pub next_replacement: Vec<usize>,
    pub heap: NodeHeap,
}

impl IndexedPopulation {
    pub fn new(
        popsize: SignedInteger,
        genome_length: LargeSignedInteger,
    ) -> Result<Self, InlineAncestryError> {
        if genome_length > 0 {
            let mut nodes = NodeTable::default();

            let mut alive_nodes = vec![];
            for i in 0..popsize {
                let node = nodes.new_birth(0, genome_length);
                match node {
                    Ok(v) => assert_eq!(v, i as usize),
                    Err(v) => panic!("{}", v), // Should be an error.
                }
                alive_nodes.push(i as usize);
            }

            Ok(Self {
                nodes,
                genome_length,
                alive_nodes,
                births: vec![],
                deaths: vec![],
                next_replacement: vec![],
                heap: NodeHeap::default(),
            })
        } else {
            Err(InlineAncestryError::InvalidGenomeLength { l: genome_length })
        }
    }

    fn add_birth(&mut self, birth_time: LargeSignedInteger) -> Result<usize, usize> {
        match self.nodes.new_birth(birth_time, self.genome_length) {
            Ok(b) => Ok(b),
            Err(b) => Err(b),
        }
    }

    fn kill(&mut self, index: usize) {
        assert!(index < self.nodes.counts.len());
        self.nodes.flags[index].clear_alive();
        self.nodes.ancestry[index].retain(|a| {
            if a.left() == 0 && a.right() == self.genome_length {
                false
            } else {
                true
            }
        });
    }

    fn propagate_ancestry_changes(
        &mut self,
        current_time_point: LargeSignedInteger,
    ) -> Result<(), InlineAncestryError> {
        // Set all counts to zero == setting all output node IDs to NULL.
        self.nodes.counts.fill(0);

        // and its API should have a "set number of nodes" function
        // println!("{:?}", self.heap);
        // println!("{:?}", self.nodes.flags);
        while let Some(node) = self.heap.pop() {
            println!("{} -> {:?}", current_time_point, node);
            if matches!(node.node_type, NodeType::Death) {
                self.kill(node.index);
            }
            if matches!(node.node_type, NodeType::Birth) {
                assert!(self.nodes.flags[node.index].is_alive());
            }
            println!(
                "before: {} -> {:?}, {:?}",
                node.index, self.nodes.ancestry[node.index], self.nodes.children[node.index],
            );
            let changed = crate::indexed_node_update_ancestry::update_ancestry(
                node.index,
                &self.nodes.flags,
                &mut self.nodes.ancestry,
                &mut self.nodes.parents,
                &mut self.nodes.children,
            );
            println!(
                "after: {} -> {:?}, {:?}",
                node.index, self.nodes.ancestry[node.index], self.nodes.children[node.index],
            );
            // TODO: is this the right criterion?
            // TODO: is this the right place to do this?
            //if !self.nodes.ancestry[node.index].is_empty() {
            //    self.nodes.counts[node.index] += 1;
            //    for child in self.nodes.children[node.index].keys() {
            //        self.nodes.counts[*child] += 1;
            //    }
            //    println!("{} {}", node.index, self.nodes.counts[node.index]);
            //}
            if self.nodes.flags[node.index].is_alive() {
                self.nodes.counts[node.index] += 1;
            }
            for child in self.nodes.children[node.index].keys() {
                assert!(self.nodes.parents[*child].contains(&node.index));
                //assert!(!self.nodes.ancestry[node.index].is_empty());
                println!(
                    "incrementing counts of {} and {} <-> {:?}",
                    node.index, *child, self.nodes.parents[*child]
                );
                self.nodes.counts[node.index] += 1;
                //self.nodes.counts[*child] += 1;
            }

            // This assert is wrong, as it catches unary
            // transmissions as something we should be keeping.

            //if self.nodes.children[node.index].is_empty()
            //    && !self.nodes.flags[node.index].is_alive()
            //{
            //    assert!(
            //        self.nodes.ancestry[node.index].is_empty(),
            //        "{:?} -> {} -> {:?}",
            //        self.nodes.parents[node.index],
            //        node.index,
            //        self.nodes.ancestry[node.index]
            //    );
            //}

            #[cfg(debug_assertions)]
            {
                if self.nodes.ancestry[node.index].is_empty() {
                    assert!(self.nodes.children[node.index].is_empty());
                    for (i, p) in self.nodes.parents.iter().enumerate() {
                        assert!(p.get(&node.index).is_none(), "{} <-> {}", i, node.index);
                    }
                }
            }

            if changed || self.nodes.flags[node.index].is_alive() {
                for parent in self.nodes.parents[node.index].iter() {
                    println!(
                        "{}: adding parent node {} to heap",
                        current_time_point, *parent
                    );
                    assert_ne!(*parent, node.index);
                    self.heap
                        .push_if(*parent, self.nodes.birth_time[*parent], NodeType::Parent);
                }
            }
        }
        let mut x = vec![0; self.nodes.counts.len()];
        for (i, parents) in self.nodes.parents.iter().enumerate() {
            if self.nodes.flags[i].is_alive() {
                x[i] += 1;
            }
            for p in parents {
                x[*p] += 1;
            }
        }
        println!("{} {}", x.len(), x.iter().filter(|i| **i > 0).count());
        println!("The ancestry is:");
        for (i, a) in self.nodes.ancestry.iter().enumerate() {
            println!("{} -> {}, {:?}", i, x[i], *a);
        }
        // println!("{:?}", self.nodes);
        // println!("{:?}", self.nodes.flags);
        Ok(())
    }

    fn simplify(
        &mut self,
        current_time_point: LargeSignedInteger,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "1096 starting simplifying time point {}",
            current_time_point
        );
        //assert_eq!(self.deaths.len(), self.births.len()); // NOTE: this is wrong for growing pops, etc..
        self.heap.initialize(self.nodes.birth_time.len());

        for b in self.births.iter() {
            assert_eq!(self.nodes.birth_time[*b], current_time_point);
            assert!(self.nodes.flags[*b].is_alive());
            println!("{} adding birth node {} to heap", current_time_point, *b);
            self.heap
                .push_if(*b, self.nodes.birth_time[*b], NodeType::Birth);
        }
        for d in self.deaths.iter() {
            println!("{} adding death node {} to heap", current_time_point, *d);
            self.heap
                .push_if(*d, self.nodes.birth_time[*d], NodeType::Death);
        }
        self.births.clear();

        // println!("{:?}", self.nodes.flags);
        self.propagate_ancestry_changes(current_time_point)?;
        // println!("{:?}", self.nodes.flags);

        debug_assert_eq!(
            self.nodes.flags.iter().filter(|x| x.is_alive()).count(),
            self.alive_nodes.len(),
            "{}",
            current_time_point
        );

        // add to queue
        // We clear the queue to avoid duplicating
        // indexes (e.g., an index previously entered
        // but did not get recycled in the last round).
        self.nodes.queue.clear();
        let mut reachable = 0;
        for (i, c) in self.nodes.counts.iter().enumerate() {
            if *c == 0 {
                // println!("setting {} for recycling", i);
                self.nodes.queue.push(i);
            } else {
                reachable += 1;
            }
        }
        println!(
            "{} {} {}",
            current_time_point,
            reachable,
            self.nodes.queue.len()
        );

        #[cfg(debug_assertions)]
        {
            for (i, p) in self.nodes.parents.iter().enumerate() {
                if self.nodes.counts[i] > 0 {
                    //for pi in p {
                    //    assert!(self.nodes.counts[*pi] > 0, "{}", *pi);
                    //    for c in self.nodes.children[*pi].keys() {
                    //        assert!(self.nodes.counts[*c] > 0, "{}", *c);
                    //    }
                    //}
                    if self.nodes.ancestry[i].is_empty() {
                        assert!(!self.nodes.flags[i].is_alive());
                        // node i cannot be a parent
                        for (j, pp) in self.nodes.parents.iter().enumerate() {
                            assert!(pp.get(&i).is_none());
                        }
                        // it probably shouldn't be a child?
                        for (j, pp) in self.nodes.children.iter().enumerate() {
                            assert!(pp.get(&i).is_none());
                        }

                        assert_eq!(
                            self.nodes.counts[i],
                            0,
                            "{} | {} {} | {:?} {:?} | {:?}",
                            i,
                            current_time_point,
                            self.nodes.birth_time[i],
                            *p,
                            self.nodes.children[i],
                            self.nodes.flags[i],
                        );
                    } else {
                        if self.nodes.counts[i] == 0 {
                            println!("I don't like this");
                            for c in self.nodes.children[i].keys() {
                                println!("{:?}", self.nodes.parents[*c]);
                            }
                        }
                        assert!(
                            self.nodes.counts[i] > 0,
                            "{} => {:?}",
                            i,
                            self.nodes.ancestry[i]
                        );
                    }
                }
            }
        }

        assert!(self.heap.heap.is_empty());

        println!("1096 done simplifying time point {}", current_time_point);

        Ok(())
    }
}

impl EvolveAncestry for IndexedPopulation {
    fn genome_length(&self) -> LargeSignedInteger {
        self.genome_length
    }

    fn setup(&mut self, _final_time: LargeSignedInteger) {}

    fn generate_deaths(&mut self, death: &mut neutral_evolution::Death) -> usize {
        self.deaths.clear();
        self.next_replacement.clear();

        for (i, n) in self.alive_nodes.iter().enumerate() {
            if death.dies() {
                self.deaths.push(*n);
                self.next_replacement.push(i);
            }
        }

        self.next_replacement.len()
    }

    fn current_population_size(&self) -> usize {
        self.alive_nodes.len()
    }

    // NOTE: having to implement this
    // trait forces us to change the api of the
    // population to accommodate the behavior.
    fn record_birth(
        &mut self,
        birth_time: LargeSignedInteger,
        _final_time: LargeSignedInteger,
        breakpoints: &[neutral_evolution::TransmittedSegment],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let birth_node_index = self.add_birth(birth_time).unwrap();

        for b in breakpoints {
            let parent = b.parent; // self.alive_nodes[b.parent];
            assert!(
                self.nodes.birth_time[parent] < birth_time,
                "{} ({}) -> {} ({})",
                parent,
                self.nodes.birth_time[parent],
                birth_node_index,
                birth_time
            );
            assert_ne!(birth_node_index, parent);
            println!(
                "adding parent {} (or is it {}) to {} | {:?}, {:?}",
                b.parent,
                self.alive_nodes[b.parent],
                birth_node_index,
                self.next_replacement,
                self.alive_nodes
            );
            self.nodes.parents[birth_node_index].insert(parent);
            self.nodes
                .add_child_segment(b.left, b.right, parent, birth_node_index)
                .unwrap();
        }

        // handle our updating of alive nodes
        match self.next_replacement.pop() {
            Some(index) => {
                // println!(
                //     "replacing death {} at {} with {}",
                //     self.alive_nodes[index], index, birth_node_index
                // );
                self.alive_nodes[index] = birth_node_index;
            }
            None => {
                // println!("pushing alive node {}", birth_node_index);
                self.alive_nodes.push(birth_node_index);
            }
        }
        self.births.push(birth_node_index);

        Ok(())
    }

    fn simplify(
        &mut self,
        current_time_point: LargeSignedInteger,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // println!("about to simplify at {}", current_time_point);
        // println!("{:?}", self.deaths);
        // println!("{:?}", self.births);
        // println!("{:?}", self.alive_nodes);
        self.simplify(current_time_point)
    }

    fn finish(
        &mut self,
        current_time_point: LargeSignedInteger,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

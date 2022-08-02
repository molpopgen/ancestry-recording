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
pub struct NodeHeap(BinaryHeap<PrioritizedNode>);

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
            Ok(b) => {
                self.births.push(b);
                Ok(b)
            }
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

    fn propagate_ancestry_changes(&mut self) -> Result<(), InlineAncestryError> {
        // Set all counts to zero == setting all output node IDs to NULL.
        self.nodes.counts.fill(0);
        // NOTE: this vector should be stored as part of the queue
        // and its API should have a "set number of nodes" function
        let mut node_in_queue = vec![false; self.nodes.counts.len()];
        while let Some(node) = self.heap.0.pop() {
            node_in_queue[node.index] = false;
            if matches!(node.node_type, NodeType::Death) {
                self.kill(node.index);
            }
            let changed = crate::indexed_node_update_ancestry::update_ancestry(
                node.index,
                &self.nodes.flags,
                &mut self.nodes.ancestry,
                &mut self.nodes.parents,
                &mut self.nodes.children,
            );
            // TODO: is this the right criterion?
            // TODO: is this the right place to do this?
            if !self.nodes.ancestry[node.index].is_empty() {
                self.nodes.counts[node.index] += 1;
                for child in self.nodes.children[node.index].keys() {
                    self.nodes.counts[*child] += 1;
                }
            }
            if changed || self.nodes.flags[node.index].is_alive() {
                for parent in self.nodes.parents[node.index].iter() {
                    if !node_in_queue[*parent] {
                        self.heap.0.push(PrioritizedNode {
                            index: *parent,
                            birth_time: self.nodes.birth_time[*parent],
                            node_type: NodeType::Parent,
                        });
                        node_in_queue[*parent] = true;
                    }
                }
            }
        }
        println!("{:?}", self.nodes);
        Ok(())
    }

    fn simplify(
        &mut self,
        current_time_point: LargeSignedInteger,
    ) -> Result<(), Box<dyn std::error::Error>> {
        //assert_eq!(self.deaths.len(), self.births.len()); // NOTE: this is wrong for growing pops, etc..
        assert!(self.heap.0.is_empty());

        for b in self.births.iter() {
            assert_eq!(self.nodes.birth_time[*b], current_time_point);
            assert!(self.nodes.flags[*b].is_alive());
            self.heap.0.push(PrioritizedNode {
                index: *b,
                birth_time: self.nodes.birth_time[*b],
                node_type: NodeType::Birth,
            });
        }
        for d in self.deaths.iter() {
            self.heap.0.push(PrioritizedNode {
                index: *d,
                birth_time: self.nodes.birth_time[*d],
                node_type: NodeType::Death,
            });
        }
        self.births.clear();

        self.propagate_ancestry_changes()?;

        // add to queue
        // We clear the queue to avoid duplicating
        // indexes (e.g., an index previously entered
        // but did not get recycled in the last round).
        self.nodes.queue.clear();
        let mut reachable = 0;
        for (i, c) in self.nodes.counts.iter().enumerate() {
            if *c == 0 {
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

        assert!(self.heap.0.is_empty());

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

        for i in 0..self.alive_nodes.len() {
            if death.dies() {
                self.deaths.push(i);
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
            self.nodes.parents[birth_node_index].insert(b.parent);
            self.nodes
                .add_child_segment(b.left, b.right, b.parent, birth_node_index)
                .unwrap();
        }

        // handle our updating of alive nodes
        match self.next_replacement.pop() {
            Some(index) => {
                self.alive_nodes[index] = birth_node_index;
            }
            None => self.alive_nodes.push(birth_node_index),
        }
        self.births.push(birth_node_index);

        Ok(())
    }

    fn simplify(
        &mut self,
        current_time_point: LargeSignedInteger,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.simplify(current_time_point)
    }

    fn finish(
        &mut self,
        current_time_point: LargeSignedInteger,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

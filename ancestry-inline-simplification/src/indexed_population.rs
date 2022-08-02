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

    fn propagate_ancestry_changes(&mut self) -> Result<(), InlineAncestryError> {
        // Set all counts to zero == setting all output node IDs to NULL.
        self.nodes.counts.fill(0);

        // and its API should have a "set number of nodes" function
        // println!("{:?}", self.heap);
        // println!("{:?}", self.nodes.flags);
        while let Some(node) = self.heap.pop() {
            println!("{:?}", node);
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
                println!("{} {}", node.index, self.nodes.counts[node.index]);
            }

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
                    assert_ne!(*parent, node.index);
                    self.heap
                        .push_if(*parent, self.nodes.birth_time[*parent], NodeType::Parent);
                }
            }
        }
        // println!("{:?}", self.nodes);
        // println!("{:?}", self.nodes.flags);
        Ok(())
    }

    fn simplify(
        &mut self,
        current_time_point: LargeSignedInteger,
    ) -> Result<(), Box<dyn std::error::Error>> {
        //assert_eq!(self.deaths.len(), self.births.len()); // NOTE: this is wrong for growing pops, etc..
        self.heap.initialize(self.nodes.birth_time.len());

        for b in self.births.iter() {
            assert_eq!(self.nodes.birth_time[*b], current_time_point);
            assert!(self.nodes.flags[*b].is_alive());
            self.heap
                .push_if(*b, self.nodes.birth_time[*b], NodeType::Birth);
        }
        for d in self.deaths.iter() {
            self.heap
                .push_if(*d, self.nodes.birth_time[*d], NodeType::Death);
        }
        self.births.clear();

        // println!("{:?}", self.nodes.flags);
        self.propagate_ancestry_changes()?;
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
                println!("setting {} for recycling", i);
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
                for pi in p {
                    assert!(self.nodes.counts[*pi] > 0, "{}", *pi);
                    for c in self.nodes.children[*pi].keys() {
                        assert!(self.nodes.counts[*c] > 0, "{}", *c);
                    }
                }
                if self.nodes.ancestry[i].is_empty() {
                    assert_eq!(self.nodes.counts[i], 0);
                } else {
                    assert!(self.nodes.counts[i] > 0);
                }
            }
        }

        assert!(self.heap.heap.is_empty());

        println!("done simplifying time point {}", current_time_point);

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
            let parent = self.alive_nodes[b.parent];
            assert_ne!(birth_node_index, parent);
            println!(
                "adding parent {} (or is it {}) to {}",
                b.parent, self.alive_nodes[b.parent], birth_node_index
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

use crate::node::Node;
use crate::node_heap::NodeHeap;
use crate::InlineAncestryError;
use crate::LargeSignedInteger;
use crate::SignedInteger;
use hashbrown::HashSet;
use neutral_evolution::EvolveAncestry;

pub struct Population {
    next_node_id: SignedInteger,
    genome_length: LargeSignedInteger,
    replacements: Vec<usize>,
    births: Vec<Node>,
    next_replacement: usize,
    node_heap: NodeHeap,
    pub nodes: Vec<Node>,
}

impl Population {
    pub fn new(
        popsize: SignedInteger,
        genome_length: LargeSignedInteger,
    ) -> Result<Self, InlineAncestryError> {
        if genome_length > 0 {
            let next_node_id = popsize;

            let mut nodes = vec![];

            for i in 0..next_node_id {
                let node = Node::new_alive_with_ancestry_mapping_to_self(i, 0, genome_length);
                nodes.push(node);
            }

            Ok(Self {
                next_node_id,
                genome_length,
                replacements: vec![],
                births: vec![],
                next_replacement: 0,
                node_heap: NodeHeap::default(),
                nodes,
            })
        } else {
            Err(InlineAncestryError::InvalidGenomeLength { l: genome_length })
        }
    }

    pub fn birth(&mut self, birth_time: LargeSignedInteger) -> Node {
        assert!(birth_time >= 0);
        let index = self.next_node_id;
        self.next_node_id += 1;
        Node::new_alive_with_ancestry_mapping_to_self(index, birth_time, self.genome_length)
    }

    pub fn get(&self, who: usize) -> Option<&Node> {
        self.nodes.get(who)
    }

    pub fn get_mut(&mut self, who: usize) -> Option<&mut Node> {
        self.nodes.get_mut(who)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn all_reachable_nodes(&self) -> HashSet<Node> {
        crate::util::all_reachable_nodes(&self.nodes)
    }

    pub fn num_still_reachable(&self) -> usize {
        self.all_reachable_nodes().len()
    }

    pub fn validate_graph(&self) -> Result<(), InlineAncestryError> {
        crate::util::validate_graph(&self.nodes)
    }
}

impl EvolveAncestry for Population {
    fn genome_length(&self) -> LargeSignedInteger {
        self.genome_length
    }

    fn generate_deaths(&mut self, death: &mut neutral_evolution::Death) -> usize {
        self.replacements.clear();
        self.next_replacement = 0;

        for i in 0..self.nodes.len() {
            if death.dies() {
                self.replacements.push(i);
            }
        }

        self.replacements.len()
    }

    fn current_population_size(&self) -> usize {
        self.nodes.len()
    }

    fn record_birth(
        &mut self,
        birth_time: LargeSignedInteger,
        breakpoints: &[neutral_evolution::TransmittedSegment],
    ) -> Result<(), Box<dyn std::error::Error>> {
        assert!(!breakpoints.is_empty());
        // Give birth to a new Individual ("node")
        let mut birth = self.birth(birth_time);

        for b in breakpoints {
            // Increase ref count of parent
            let mut parent = self.get_mut(b.parent).as_mut().unwrap().clone();

            // Add references to birth for each segment
            parent.add_child_segment(b.left, b.right, birth.clone())?;
            // MOVE parent w/o increasing ref count
            birth.add_parent(parent)?;
        }

        assert!(!birth.borrow().parents.is_empty());

        // MOVE the birth w/o increasing ref count
        self.births.push(birth);
        Ok(())
    }

    fn simplify(
        &mut self,
        current_time_point: LargeSignedInteger,
    ) -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(self.replacements.len(), self.births.len());
        assert!(self.node_heap.is_empty());

        let ndeaths = self.replacements.len();

        for death in 0..ndeaths {
            let dead = self.nodes[death].clone();
            let birth = self.births[death].clone();
            assert_eq!(self.births[death].borrow().birth_time, current_time_point);
            assert!(self.nodes[death].is_alive());
            self.node_heap.push_death(dead)?;
            self.node_heap.push_birth(birth.clone())?;

            self.nodes[death] = birth;
        }

        self.births.clear();

        crate::propagate_ancestry_changes::propagate_ancestry_changes(
            self.genome_length,
            &mut self.node_heap,
        )?;

        #[cfg(debug_assertions)]
        {
            self.validate_graph()?;
        }

        assert!(self.node_heap.is_empty());
        Ok(())
    }

    fn finish(
        &mut self,
        _current_time_point: LargeSignedInteger,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

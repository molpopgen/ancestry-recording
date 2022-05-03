use crate::node::Node;
use crate::segments::HalfOpenInterval;
use crate::InlineAncestryError;
use crate::LargeSignedInteger;
use crate::SignedInteger;
use neutral_evolution::EvolveAncestry;

pub struct Population {
    next_node_id: SignedInteger,
    genome_length: LargeSignedInteger,
    replacements: Vec<usize>,
    births: Vec<Node>,
    next_replacement: usize,
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

    pub fn kill(&mut self, who: usize) {
        let genome_length = self.genome_length;
        if let Some(node) = self.get_mut(who) {
            node.borrow_mut().flags.clear_alive();
            node.borrow_mut().ancestry.retain(|a| {
                if a.left() == 0 && a.right() == genome_length {
                    false
                } else {
                    true
                }
            });
        } else {
            panic!("{who} out of range for kill");
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn num_still_reachable(&self) -> usize {
        let mut reachable = hashbrown::HashSet::new();

        for node in &self.nodes {
            let mut stack = vec![node.clone()];
            while let Some(popped) = stack.pop() {
                reachable.insert(popped.clone());
                for parent in &popped.borrow().parents {
                    if !reachable.contains(parent) {
                        stack.push(parent.clone());
                    }
                }
            }
        }

        reachable.len()
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

        let ndeaths = self.replacements.len();

        for death in 0..ndeaths {
            let mut dead = self.nodes[death].clone();
            // FIXME: this should be an Individual fn
            // FIXME: the name kill should be changed
            self.kill(death);
            assert!(!dead.is_alive());
            dead.propagate_upwards()?;
            assert_eq!(self.births[death].borrow().birth_time, current_time_point);

            // NOTE: The following assertion is WRONG!
            // A parent is likely to be unary w.r.to a given
            // birth, and the previous call to propagate_upwards
            // will remove that branch if the parent is dead.
            // assert!(!self.births[death].borrow().parents.is_empty());
            self.nodes[death] = self.births[death].clone();
            assert!(self.nodes[death].is_alive());
        }

        for b in self.births.iter_mut() {
            // NOTE: see previous note
            // assert!(!b.borrow().parents.is_empty());
            b.propagate_upwards()?;
        }

        self.births.clear();

        Ok(())
    }

    fn finish(
        &mut self,
        _current_time_point: LargeSignedInteger,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

use ancestry_common::LargeSignedInteger;
use neutral_evolution::EvolveAncestry;
use tskit::prelude::*;
use tskit::TableCollection;

pub struct EvolvableTableCollection {
    tables: TableCollection,
    alive_nodes: Vec<NodeId>,
    idmap: Vec<NodeId>,
    popsize: usize,
    replacements: Vec<usize>,
    births: Vec<NodeId>,
    bookmark: tskit::types::Bookmark,
    simplification_interval: LargeSignedInteger,
    last_time_simplified: Option<LargeSignedInteger>, // TODO: do we really need this?
}

impl EvolvableTableCollection {
    pub fn new(
        sequence_length: tskit::Position,
        popsize: usize,
        simplification_interval: LargeSignedInteger,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut tables = TableCollection::new(sequence_length)?;
        let mut alive_nodes = vec![];

        for i in 0..popsize {
            let id = tables.add_node(0, 0.0, PopulationId::NULL, IndividualId::NULL)?;
            alive_nodes.push(id);
        }

        Ok(Self {
            tables,
            alive_nodes,
            idmap: vec![],
            popsize,
            replacements: vec![],
            births: vec![],
            bookmark: tskit::types::Bookmark::new(),
            simplification_interval,
            last_time_simplified: None,
        })
    }

    fn adjust_node_times(&mut self, delta: LargeSignedInteger) {
        let delta = delta as f64;
        let tables = self.tables.as_mut_ptr();

        let num_nodes = usize::from(self.tables.nodes().num_rows());
        let node_offset = self.bookmark.offsets.nodes as usize;
        let time = unsafe { std::slice::from_raw_parts_mut((*tables).nodes.time, num_nodes) };
        let max_time = *time.last().unwrap();

        for t in time.iter_mut().take(node_offset) {
            *t += delta;
        }
        for t in time.iter_mut().skip(node_offset) {
            *t -= max_time;
            *t *= -1.0;
        }
    }

    fn enact_replacements(&mut self) {
        if !self.births.is_empty() {
            assert_eq!(self.replacements.len(), self.births.len());
            for (r, b) in self.replacements.iter().zip(self.births.iter()) {
                self.alive_nodes[*r] = *b;
            }
        }
        self.births.clear();
    }

    fn simplify_details(
        &mut self,
        current_time_point: LargeSignedInteger,
        force: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.enact_replacements();
        if current_time_point > 0
            && (force || current_time_point % self.simplification_interval == 0)
        {
            let delta = match self.last_time_simplified {
                Some(d) => current_time_point - d,
                None => current_time_point,
            };
            self.adjust_node_times(delta);
            self.tables
                .sort(&self.bookmark, tskit::TableSortOptions::default())?;

            self.tables
                .check_integrity(tskit::TableIntegrityCheckFlags::CHECK_EDGE_ORDERING)?;

            if self.bookmark.offsets.edges > 0 {
                // To simplify, the edge table must
                // have the newest edges at the front.
                // Sorting using a bookmark defines where
                // to start sorting FROM.  So, we need to rotate
                // each column

                let num_edges = usize::from(self.tables.edges().num_rows());

                // Get the raw pointer to the tsk_table_collection_t
                let table_ptr = self.tables.as_mut_ptr();

                let offset = usize::try_from(self.bookmark.offsets.edges)?;

                // SAFETY: the tskit::TableCollection does not
                // allow the managed pointer to be NULL
                unsafe {
                    // For each column (that we are using), put the newest edges at the front.
                    rotate_left((*table_ptr).edges.parent, num_edges, offset);
                    rotate_left((*table_ptr).edges.child, num_edges, offset);
                    rotate_left((*table_ptr).edges.left, num_edges, offset);
                    rotate_left((*table_ptr).edges.right, num_edges, offset);
                }
            }
            let idmap = match self.tables.simplify(
                &self.alive_nodes,
                tskit::SimplificationOptions::default(),
                true,
            ) {
                Err(e) => return Err(Box::new(e)),
                Ok(x) => x.unwrap(),
            };
            self.last_time_simplified = Some(current_time_point);

            // next time, we will only sort the new edges
            // TODO: try to restore this
            // self.bookmark.offsets.edges = u64::from(self.tables.edges().num_rows());
            // for adjusting time.
            self.bookmark.offsets.nodes = u64::from(self.tables.nodes().num_rows());

            // remap the alive nodes
            for alive in self.alive_nodes.iter_mut() {
                *alive = idmap[usize::from(*alive)];
                assert!(!alive.is_null());
            }
            Ok(())
        } else {
            Ok(())
        }
    }
}

unsafe fn rotate_left<T>(data: *mut T, len: usize, mid: usize) {
    let s = std::slice::from_raw_parts_mut(data, len);
    s.rotate_left(mid);
}

impl TryFrom<EvolvableTableCollection> for tskit::TreeSequence {
    type Error = tskit::TskitError;
    fn try_from(value: EvolvableTableCollection) -> Result<Self, Self::Error> {
        value
            .tables
            .tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES)
    }
}

impl EvolveAncestry for EvolvableTableCollection {
    fn generate_deaths(&mut self, death: &mut neutral_evolution::Death) -> usize {
        self.replacements.clear();
        for i in 0..self.alive_nodes.len() {
            if death.dies() {
                self.replacements.push(i);
            }
        }
        self.replacements.len()
    }

    fn current_population_size(&self) -> usize {
        self.popsize
    }

    fn record_birth(
        &mut self,
        birth_time: LargeSignedInteger,
        breakpoints: &[neutral_evolution::TransmittedSegment],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let child = self.tables.add_node(
            0,
            Time::from(birth_time as f64),
            PopulationId::NULL,
            IndividualId::NULL,
        )?;
        for b in breakpoints {
            self.tables.add_edge(
                Position::from(b.left as f64),
                Position::from(b.right as f64),
                self.alive_nodes[b.parent],
                child,
            )?;
        }
        self.births.push(child);

        Ok(())
    }

    fn simplify(
        &mut self,
        current_time_point: LargeSignedInteger,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.simplify_details(current_time_point, false)
    }

    fn finish(
        &mut self,
        current_time_point: LargeSignedInteger,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let rv = match self.last_time_simplified {
            Some(x) => {
                if x != current_time_point {
                    self.simplify_details(current_time_point, true)
                } else {
                    Ok(())
                }
            }
            None => self.simplify_details(current_time_point, true),
        };
        rv
    }
}

#[cfg(test)]
mod tests {}

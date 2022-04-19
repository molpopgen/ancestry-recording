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

        unsafe {
            let time = std::slice::from_raw_parts_mut(
                (*tables).nodes.time,
                usize::from(self.tables.nodes().num_rows()),
            );
            for t in time.iter_mut() {
                let before = *t;
                *t *= -1.0;
                *t -= delta;
                *t *= -1.0;
                println!("convert {} -> {}",before, *t);
            }
            for t in time {
                println!("converted {}", *t);
            }
        }
    }
}

unsafe fn rotate_left<T>(data: *mut T, len: usize, mid: usize) {
    let s = std::slice::from_raw_parts_mut(data, len);
    s.rotate_left(mid);
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
        // TODO: sort tables, update node id mappings, replace death nodes with birth in
        // alive_nodes.
        // Also, need to deal with bookmarking the last time simplified so that we can be more
        // efficient than simply sorting the entire table collection.

        if current_time_point > 0 && current_time_point % self.simplification_interval == 0 {
            for i in self.tables.nodes().iter() {
                println!("{}",i.time);
            }
            let delta = match self.last_time_simplified {
                Some(d) => current_time_point - d,
                None => current_time_point,
            };
            self.adjust_node_times(delta);
            for i in self.tables.nodes().iter() {
                println!("after {} {}",i.time, delta);
            }
            self.tables
                .sort(&self.bookmark, tskit::TableSortOptions::default())?;
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
            self.bookmark.offsets.edges = u64::from(self.tables.edges().num_rows());

            // remap the alive nodes
            for (i, j) in self.alive_nodes.iter_mut().enumerate() {
                *j = idmap[i];
            }
            Ok(())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {}

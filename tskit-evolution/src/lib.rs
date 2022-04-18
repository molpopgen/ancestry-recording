use ancestry_common::LargeSignedInteger;
use neutral_evolution::NeutralEvolution;
use tskit::prelude::*;
use tskit::TableCollection;

pub struct EvolvableTableCollection {
    tables: TableCollection,
    alive_nodes: Vec<NodeId>,
    popsize: usize,
    replacements: Vec<usize>,
    births: Vec<NodeId>,
    simplification_interval: LargeSignedInteger,
    last_time_simplified: Option<LargeSignedInteger>,
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
            popsize,
            replacements: vec![],
            births: vec![],
            simplification_interval,
            last_time_simplified: None,
        })
    }
}

impl NeutralEvolution for EvolvableTableCollection {
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

        Ok(())
    }
}

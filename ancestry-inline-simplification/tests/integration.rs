use ancestry_inline_simplification::*;
use neutral_evolution::{evolve, Parameters};
use tskit::prelude::*;
use tskit_evolution::EvolvableTableCollection;

#[test]
fn test_simulation_round_trip_nonoverlapping_gens() {
    for seed in [101, 201, 301, 401, 8512389, 12853581239, 95192] {
        // number of haploids, genome length
        let mut pop = Population::new(10, 100).unwrap();
        let mut evolveable_tables =
            tskit_evolution::EvolvableTableCollection::new(100, 10, 10).unwrap();
        // death rate, mean no. crossovers, no. steps to sim
        let p = Parameters::new(1.0, 1e-3, 250).unwrap();
        evolve(seed, p, &mut pop).unwrap();
        evolve(seed, p, &mut evolveable_tables).unwrap();
        assert!(pop.nodes.iter().any(|i| i.borrow().parents.len() > 0));
        for i in pop.nodes.iter() {
            assert_eq!(i.borrow().birth_time, 250);
            let mut stack = vec![i.clone()];
            while let Some(node) = stack.pop() {
                for p in &node.borrow().parents {
                    stack.push(p.clone());
                }
            }
        }
        let r = pop.all_reachable_nodes();
        let tables = tskit::TableCollection::from(evolveable_tables);
        assert_eq!(r.len(), usize::from(tables.nodes().num_rows()));
    }
}

#[test]
fn test_simulation_round_trip_overlapping_gens() {
    for seed in [101, 201, 301, 401, 8512389, 12853581239, 95192] {
        for pdeath in [0.25, 0.5, 0.75, 0.9] {
            let mut pop = Population::new(10, 100).unwrap();
            let mut evolveable_tables =
                tskit_evolution::EvolvableTableCollection::new(100, 10, 10).unwrap();
            let p = Parameters::new(pdeath, 1e-1, 250).unwrap();
            evolve(seed, p, &mut pop).unwrap();
            evolve(seed, p, &mut evolveable_tables).unwrap();
            assert!(pop.nodes.iter().any(|i| i.borrow().parents.len() > 0));
            let r = pop.all_reachable_nodes();
            let tables = tskit::TableCollection::from(evolveable_tables);
            assert_eq!(r.len(), usize::from(tables.nodes().num_rows()));
        }
    }
}

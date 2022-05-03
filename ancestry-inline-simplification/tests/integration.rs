use ancestry_inline_simplification::*;
use neutral_evolution::{evolve, Parameters};

#[test]
fn test_simulation_round_trip() {
    // number of haploids, genome length
    let mut pop = Population::new(10, 100).unwrap();
    // death rate, mean no. crossovers, no. steps to sim
    let p = Parameters::new(1.0, 1e-3, 100).unwrap();
    evolve(101, p, &mut pop).unwrap();
    assert!(pop.nodes.iter().any(|i| i.borrow().parents.len() > 0));
    for i in pop.nodes.iter() {
        assert_eq!(i.borrow().birth_time, 100);
        let mut stack = vec![i.clone()];
        while let Some(node) = stack.pop() {
            for p in &node.borrow().parents {
                stack.push(p.clone());
            }
        }
    }
}

#[test]
fn test_simulation_round_trip_overlapping_gens() {
    for pdeath in [0.25, 0.5, 0.75, 0.9] {
        let mut pop = Population::new(10, 100).unwrap();
        let p = Parameters::new(pdeath, 1e-1, 100).unwrap();
        evolve(101, p, &mut pop).unwrap();
        assert!(pop.nodes.iter().any(|i| i.borrow().parents.len() > 0));
    }
}

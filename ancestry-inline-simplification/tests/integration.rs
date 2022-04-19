use ancestry_inline_simplification::*;
use neutral_evolution::{evolve, Parameters};

#[test]
fn test_simulation_round_trip() {
    for simplification_interval in 1..6_i64 {
        let mut pop = Population::new(10, 100);
        let p = Parameters::new(1.0, 1e-3, 100, 100).unwrap();
        evolve([101, 202], p, &mut pop).unwrap();
    }
}

#[test]
fn test_simulation_round_trip_overlapping_gens() {
    for pdeath in [0.25, 0.5, 0.75, 0.9] {
        for simplification_interval in 1..6_i64 {
            let mut pop = Population::new(10, 100);
            let p = Parameters::new(pdeath, 1e-1, 100, 100).unwrap();
            evolve([101, 202], p, &mut pop).unwrap();
        }
    }
}

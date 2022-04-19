use neutral_evolution::{evolve, Parameters};
use tskit_evolution::*;

#[test]
fn test_simulation_round_trip() {
    for simplification_interval in 1..6_i64 {
        let mut t = EvolvableTableCollection::new(tskit::Position::from(100.0), 10, simplification_interval).unwrap();
        let p = Parameters::new(1.0, 1e-3, 100, 100).unwrap();
        evolve([101, 202], p, &mut t).unwrap();
    }
}

use crate::individual::Individual;
use crate::population::Population;
use crate::LargeSignedInteger;
use crate::SignedInteger;
use rand::distributions::Uniform;
use rand::prelude::*;
use rand_pcg::Pcg64;

pub struct SimParams {
    popsize: SignedInteger,
    num_steps: SignedInteger,
    death_prob: f64,
    genome_length: LargeSignedInteger,
}

pub fn simulate(seed: u64, params: SimParams) -> Population {
    let mut rng = Pcg64::seed_from_u64(seed);

    let breakpoint = Uniform::<LargeSignedInteger>::new(0, params.genome_length);
    let u01 = Uniform::<f64>::new(0., 1.);
    let pick_parent = Uniform::<SignedInteger>::new(0, params.popsize);

    let mut pop = Population::new(params.popsize);

    for birth_time in 1..(params.num_steps + 1) {
        let mut deaths: Vec<usize> = vec![];
        let mut replacements: Vec<Individual> = vec![];

        // main loop here

        assert_eq!(deaths.len(), replacements.len());

        for (i, r) in deaths.iter().zip(replacements.iter_mut()) {
            std::mem::swap(&mut pop.individuals[*i], &mut r.clone());
        }
    }

    pop
}

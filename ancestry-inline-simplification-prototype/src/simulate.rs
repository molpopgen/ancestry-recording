use crate::LargeSignedInteger;
use crate::SignedInteger;
use rand::prelude::*;
use rand_pcg::Pcg64;

pub fn simulate(seed: u64, popsize: SignedInteger, genome_length: LargeSignedInteger) {
    let mut rng = Pcg64::seed_from_u64(seed);

    let breakpoint = rand::distributions::Uniform::<LargeSignedInteger>::new(0, genome_length);

    let _ = breakpoint.sample(&mut rng);
}

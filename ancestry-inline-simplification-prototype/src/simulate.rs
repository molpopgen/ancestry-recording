use crate::ancestry::Ancestry;
use crate::ancestry::NodeStatus;
use crate::LargeSignedInteger;
use crate::SignedInteger;
use rgsl::rng;

fn ran_flat(rng: &mut rng::Rng, lo: f64, hi: f64) -> f64 {
    let mut rv = rng.flat(lo, hi);

    while match rv.partial_cmp(&hi) {
        Some(std::cmp::Ordering::Equal) => true,
        Some(_) => false,
        None => panic!("gsl_rng_ran_flat should not return NaN"),
    } {
        rv = rng.flat(lo, hi);
    }
    rv
}

pub fn simulate(
    seed: usize,
    popsize: SignedInteger,
    timesteps: LargeSignedInteger,
    genome_length: LargeSignedInteger,
    death_prob: f64,
) -> Ancestry {
    let ancestry = Ancestry::new(popsize, genome_length);
    let mut rng = rng::Rng::new(rng::algorithms::mt19937()).unwrap();
    rng.set(seed);

    for timestep in 0..timesteps {
        let birth_time = timestep + 1;
    }

    ancestry
}

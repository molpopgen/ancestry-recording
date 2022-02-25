use crate::ancestry::Ancestry;
use crate::ancestry::NodeStatus;
use crate::LargeSignedInteger;
use crate::SignedInteger;
use rgsl::rng;

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

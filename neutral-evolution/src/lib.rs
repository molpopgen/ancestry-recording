pub use ancestry_common::LargeSignedInteger;
use rand::prelude::Distribution;
use rand::SeedableRng;
use std::error::Error;

pub trait NeutralEvolution {
    /// Generate how many deaths (replacements) will occur at this time step.
    fn generate_deaths(&mut self, death: &mut Death) -> usize;

    /// For this prototype, we assume that pop size is
    /// constant during a sim
    fn current_population_size(&self) -> usize;

    /// Record a birth with the given parents and segments/edges.
    /// The details of generating a child are up to the specific type.
    fn record_birth(
        &mut self,
        birth_time: LargeSignedInteger,
        breakpoints: &[TransmittedSegment],
    ) -> Result<(), Box<dyn Error>>;

    fn simplify(&mut self, current_time_point: LargeSignedInteger) -> Result<(), Box<dyn Error>>;
}

pub struct Death {
    rng: rand_pcg::Pcg64,
    death_probability: f64,
    uniform: rand::distributions::Uniform<f64>,
}

impl Death {
    fn new(seed: u64, death_probability: f64) -> Self {
        Self {
            rng: rand_pcg::Pcg64::seed_from_u64(seed),
            death_probability,
            uniform: rand::distributions::Uniform::new(0., 1.),
        }
    }

    pub fn dies(&mut self) -> bool {
        self.uniform.sample(&mut self.rng) <= self.death_probability
    }
}

pub struct TransmittedSegment {
    pub left: LargeSignedInteger,
    pub right: LargeSignedInteger,
    pub parent: usize,
}

impl TransmittedSegment {
    pub fn new(left: LargeSignedInteger, right: LargeSignedInteger, parent: usize) -> Self {
        Self {
            left,
            right,
            parent,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Parameters {
    death_probability: f64,
    mean_num_crossovers: f64,
    genome_length: LargeSignedInteger,
    nsteps: LargeSignedInteger,
}

fn fill_transmissions(
    p1: usize,
    p2: usize,
    crossovers: &[LargeSignedInteger],
    transmissions: &mut Vec<TransmittedSegment>,
) {
    transmissions.clear();
    let mut p1 = p1;
    let mut p2 = p2;
    let mut last_crossover = crossovers[0];
    let mut pushed = false;
    for c in crossovers.iter().skip(1) {
        if c != &last_crossover {
            transmissions.push(TransmittedSegment {
                left: last_crossover,
                right: *c,
                parent: p1,
            });
            std::mem::swap(&mut p1, &mut p2);
            last_crossover = *c;
            pushed = true;
        } else {
            pushed = false;
        }
    }
    if !pushed {
        transmissions.push(TransmittedSegment {
            left: last_crossover,
            right: *crossovers.last().unwrap(),
            parent: p1,
        });
    }
}

fn generate_crossover_positions(
    genome_length: LargeSignedInteger,
    num_crossovers: u64,
    crossover_position: &rand_distr::Uniform<LargeSignedInteger>,
    rng: &mut rand_pcg::Pcg64,
    crossovers: &mut Vec<LargeSignedInteger>,
) {
    crossovers.clear();
    crossovers.push(0);
    for _ in 0..num_crossovers {
        let pos = crossover_position.sample(rng);
        assert!(pos > 0 && pos < genome_length);
        crossovers.push(pos);
    }
    crossovers.sort_unstable();
    crossovers.push(genome_length);
}

fn make_crossover_position_distribution(
    genome_length: LargeSignedInteger,
) -> rand_distr::Uniform<LargeSignedInteger> {
    assert!(genome_length > 0);
    rand_distr::Uniform::new(1, genome_length)
}

pub fn evolve<N: NeutralEvolution>(
    seeds: [u64; 2],
    parameters: Parameters,
    population: &mut N,
) -> Result<(), Box<dyn Error>> {
    let mut death = Death::new(seeds[0], parameters.death_probability);
    let mut rng = rand_pcg::Pcg64::seed_from_u64(seeds[1]);

    let popsize = population.current_population_size();

    let parent_picker = rand_distr::Uniform::new(0, popsize);
    let num_crossovers = rand_distr::Poisson::new(parameters.mean_num_crossovers)?;
    let crossover_position = make_crossover_position_distribution(parameters.genome_length);
    let mendel = rand_distr::Bernoulli::new(0.5).unwrap();
    let mut transmissions: Vec<TransmittedSegment> = vec![];
    let mut crossovers: Vec<LargeSignedInteger> = vec![];
    for step in 0..parameters.nsteps {
        let nreplacements = population.generate_deaths(&mut death);
        for _ in 0..nreplacements {
            let mut p1 = parent_picker.sample(&mut rng);
            let mut p2 = parent_picker.sample(&mut rng);
            if mendel.sample(&mut rng) {
                std::mem::swap(&mut p1, &mut p2);
            }
            let n = num_crossovers.sample(&mut rng) as u64;
            generate_crossover_positions(
                parameters.genome_length,
                n,
                &crossover_position,
                &mut rng,
                &mut crossovers,
            );
            fill_transmissions(p1, p2, &crossovers, &mut transmissions);
            population.record_birth(step, &transmissions)?;
        }
        population.simplify(step)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rng(seed: u64) -> rand_pcg::Pcg64 {
        rand_pcg::Pcg64::seed_from_u64(seed)
    }

    #[test]
    fn test_generate_crossover_positions() {
        let mut rng = make_rng(101);
        let genome_length = 10_i64;
        let mut crossovers = vec![];
        let crossover_position = make_crossover_position_distribution(genome_length);
        for n in 0..10_u64 {
            generate_crossover_positions(
                genome_length,
                n,
                &crossover_position,
                &mut rng,
                &mut crossovers,
            );
            assert_eq!(crossovers.len() as u64, n + 2);
            let sorted = crossovers.windows(2).all(|w| w[0] <= w[1]);
            assert!(sorted);
        }
    }

    #[test]
    fn test_fill_transmissions() {
        let p1 = 0_usize;
        let p2 = p1 + 1;
        let genome_length = 100_i64;
        let mut transmissions = vec![];

        {
            let crossovers = vec![0, 1, 3, genome_length];
            let expected_parents = vec![p1, p2, p1];
            fill_transmissions(p1, p2, &crossovers, &mut transmissions);
            for (i, t) in transmissions.iter().enumerate() {
                assert_eq!(t.parent, expected_parents[i]);
            }
            assert_eq!(transmissions.len(), 3);
        }

        {
            let crossovers = vec![0, 1, 3, 3, genome_length];
            let expected_parents = vec![p1, p2, p2];
            fill_transmissions(p1, p2, &crossovers, &mut transmissions);
            for (i, t) in transmissions.iter().enumerate() {
                assert_eq!(t.parent, expected_parents[i]);
            }
            assert_eq!(transmissions.len(), 3);
        }
    }
}

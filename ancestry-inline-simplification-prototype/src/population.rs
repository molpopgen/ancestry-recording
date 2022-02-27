use crate::individual::Individual;
use crate::SignedInteger;

pub struct Population {
    next_individual_id: SignedInteger,
    pub individuals: Vec<Individual>,
}

impl Population {
    pub fn new(popsize: SignedInteger) -> Self {
        let next_individual_id = popsize;

        let mut individuals = vec![];

        for i in 0..next_individual_id {
            individuals.push(Individual::new(i, 0));
        }

        Self {
            next_individual_id,
            individuals,
        }
    }
}

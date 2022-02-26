use crate::individual::IndividualPointer;
use crate::SignedInteger;

pub struct Population {
    next_individual_id: SignedInteger,
    pub individuals: Vec<IndividualPointer>,
}

impl Population {
    pub fn new(popsize: SignedInteger) -> Self {
        let next_individual_id = popsize;

        let mut individuals = vec![];

        for i in 0..next_individual_id {
            individuals.push(IndividualPointer::new(i, 0));
        }

        Self {
            next_individual_id,
            individuals,
        }
    }
}

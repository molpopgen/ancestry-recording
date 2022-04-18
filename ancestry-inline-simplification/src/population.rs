use crate::individual::Individual;
use crate::segments::HalfOpenInterval;
use crate::LargeSignedInteger;
use crate::SignedInteger;

pub struct Population {
    next_individual_id: SignedInteger,
    genome_length: LargeSignedInteger,
    pub individuals: Vec<Individual>,
}

impl Population {
    pub fn new(popsize: SignedInteger, genome_length: LargeSignedInteger) -> Self {
        let next_individual_id = popsize;

        let mut individuals = vec![];

        for i in 0..next_individual_id {
            individuals.push(Individual::new_alive(i, 0));
        }

        Self {
            next_individual_id,
            genome_length,
            individuals,
        }
    }

    pub fn birth(&mut self, birth_time: LargeSignedInteger) {
        assert!(birth_time >= 0);
        let index = self.next_individual_id;
        self.next_individual_id += 1;
        Individual::new_alive(index, birth_time);
    }

    pub fn get(&self, who: usize) -> Option<&Individual> {
        self.individuals.get(who)
    }

    pub fn get_mut(&mut self, who: usize) -> Option<&mut Individual> {
        self.individuals.get_mut(who)
    }

    pub fn kill(&mut self, who: usize) {
        let genome_length = self.genome_length;
        if let Some(ind) = self.get_mut(who) {
            ind.borrow_mut().flags.clear_alive();
            ind.borrow_mut().ancestry.retain(|a| {
                if a.left() == 0 && a.right() == genome_length {
                    false
                } else {
                    true
                }
            });
        } else {
            panic!("{who} out of range for kill");
        }
    }

    pub fn len(&self) -> usize {
        self.individuals.len()
    }

    pub fn is_empty(&self) -> bool {
        self.individuals.is_empty()
    }
}

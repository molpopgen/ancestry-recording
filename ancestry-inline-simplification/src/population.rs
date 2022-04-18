use crate::individual::Individual;
use crate::segments::HalfOpenInterval;
use crate::LargeSignedInteger;
use crate::SignedInteger;
use neutral_evolution::NeutralEvolution;

pub struct Population {
    next_individual_id: SignedInteger,
    genome_length: LargeSignedInteger,
    replacements: Vec<usize>,
    births: Vec<Individual>,
    next_replacement: usize,
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
            replacements: vec![],
            births: vec![],
            next_replacement: 0,
            individuals,
        }
    }

    pub fn birth(&mut self, birth_time: LargeSignedInteger) -> Individual {
        assert!(birth_time >= 0);
        let index = self.next_individual_id;
        self.next_individual_id += 1;
        Individual::new_alive(index, birth_time)
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

impl NeutralEvolution for Population {
    fn generate_deaths(&mut self, death: &mut neutral_evolution::Death) -> usize {
        self.replacements.clear();
        self.next_replacement = 0;

        for i in 0..self.individuals.len() {
            if death.dies() {
                self.replacements.push(i);
            }
        }

        self.replacements.len()
    }

    fn current_population_size(&self) -> usize {
        self.individuals.len()
    }

    fn record_birth(
        &mut self,
        birth_time: LargeSignedInteger,
        breakpoints: &[neutral_evolution::TransmittedSegment],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut birth = self.birth(birth_time);

        for b in breakpoints {
            // Increase ref count of parent
            let mut parent = self.get_mut(b.parent).as_mut().unwrap().clone();

            // Add references to birth for each segment
            parent.add_child_segment(b.left, b.right, birth.clone())?;
            // MOVE parent w/o increasing ref count
            birth.add_parent(parent)?;
        }

        // MOVE the birth w/o increasing ref count
        self.births.push(birth);
        Ok(())
    }

    fn simplify(
        &mut self,
        _current_time_point: LargeSignedInteger,
    ) -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(self.replacements.len(), self.births.len());

        let ndeaths = self.replacements.len();

        for death in 0..ndeaths {
            let mut dead_ind = self.get(death).unwrap().clone();
            self.kill(death);
            dead_ind.propagate_upwards()?;
            self.individuals[death] = self.births[death].clone();
        }

        for b in self.births.iter_mut() {
            b.propagate_upwards()?;
        }

        self.births.clear();

        Ok(())
    }
}

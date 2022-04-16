use crate::individual::Individual;
use hashbrown::HashSet;
use std::collections::BinaryHeap;

#[repr(transparent)]
struct PrioritizedIndividual(Individual);

impl PartialEq for PrioritizedIndividual {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0.as_ptr(), other.0.as_ptr())
    }
}

impl PartialOrd for PrioritizedIndividual {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.borrow().birth_time.cmp(&other.0.borrow().birth_time))
    }
}

impl Eq for PrioritizedIndividual {}

impl Ord for PrioritizedIndividual {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PrioritizedIndividual {
    fn get(self) -> Individual {
        self.0
    }
}

pub(crate) struct IndividualHeap {
    heap: BinaryHeap<PrioritizedIndividual>,
    in_heap: HashSet<Individual>,
}

impl IndividualHeap {
    pub(crate) fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            in_heap: HashSet::new(),
        }
    }

    pub fn push(&mut self, individual: Individual) {
        if !self.in_heap.contains(&individual) {
            self.in_heap.insert(individual.clone());
            self.heap.push(PrioritizedIndividual(individual));
        }
    }

    pub fn pop(&mut self) -> Option<Individual> {
        match self.heap.pop() {
            Some(x) => {
                self.in_heap.remove(&x.0);
                Some(x.get())
            }
            None => None,
        }
    }

    pub fn is_empty(&self) -> bool {
        assert_eq!(self.heap.is_empty(), self.in_heap.is_empty());
        self.heap.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_heap() {
        let a = Individual::new(0, 1);
        let b = Individual::new(0, 2);

        let mut heap = IndividualHeap::new();
        heap.push(a);
        heap.push(b);

        let mut birth_times = vec![];
        while !heap.is_empty() {
            let x = heap.pop().unwrap();
            birth_times.push(x.borrow().birth_time);
        }
        assert_eq!(birth_times, vec![2, 1]);
        assert!(heap.is_empty());
    }
}

use crate::{LargeSignedInteger, Segment, SignedInteger};
use std::cell::RefCell;
use std::rc::Rc;

// Use this over std::collections b/c the hashing
// fn is much faster. (We aren't doing cryptography.)
// TODO: See the O'Reilly book for which crate
// they recommend here.
use hashbrown::{HashMap, HashSet};

pub type ChildMap = HashMap<SignedInteger, Vec<Segment>>;

#[derive(Clone)]
pub struct Individual {
    index: SignedInteger,
    alive: bool,
    parents: Vec<IndividualPointer>,
    ancestry: Vec<Segment>,
    children: ChildMap,
}

impl Individual {
    pub fn new(index: SignedInteger) -> Self {
        Self {
            index,
            alive: true,
            parents: vec![],
            ancestry: vec![],
            children: ChildMap::default(),
        }
    }
}

type IndividualPointer = Rc<RefCell<Individual>>;

// This module is for experimenting with the Rc/RefCell pattern.
#[cfg(test)]
mod practice_tests {
    use super::*;

    fn remove_parent(parent: IndividualPointer, child: IndividualPointer) {
        child
            .borrow_mut()
            .parents
            .retain(|x| x.borrow().index != parent.borrow().index);
    }

    // Better -- does not increase ref counts just for fn call.
    fn remove_parent_via_ref(parent: &IndividualPointer, child: &IndividualPointer) {
        child
            .borrow_mut()
            .parents
            .retain(|x| x.borrow().index != parent.borrow().index);
    }

    #[test]
    fn test_interior_mutability() {
        let mut pop: Vec<IndividualPointer> = vec![];

        pop.push(IndividualPointer::new(RefCell::<Individual>::new(
            Individual::new(0),
        )));
        pop.push(IndividualPointer::new(RefCell::<Individual>::new(
            Individual::new(1),
        )));

        pop[0]
            .borrow_mut()
            .children
            .insert(1, vec![Segment::new(0, 0, 2)]);
        pop[1].borrow_mut().parents.push(pop[0].clone());
        assert_eq!(Rc::strong_count(&pop[0]), 2);
        assert_eq!(Rc::strong_count(&pop[1]), 1);

        remove_parent(pop[0].clone(), pop[1].clone());
        assert_eq!(Rc::strong_count(&pop[0]), 1);
        assert_eq!(Rc::strong_count(&pop[1]), 1);
    }

    #[test]
    fn test_interior_mutability_via_ref() {
        let mut pop: Vec<IndividualPointer> = vec![];

        pop.push(IndividualPointer::new(RefCell::<Individual>::new(
            Individual::new(0),
        )));
        pop.push(IndividualPointer::new(RefCell::<Individual>::new(
            Individual::new(1),
        )));

        pop[0]
            .borrow_mut()
            .children
            .insert(1, vec![Segment::new(0, 0, 2)]);
        pop[1].borrow_mut().parents.push(pop[0].clone());
        assert_eq!(Rc::strong_count(&pop[0]), 2);
        assert_eq!(Rc::strong_count(&pop[1]), 1);

        remove_parent_via_ref(&pop[0], &pop[1]);
        assert_eq!(Rc::strong_count(&pop[0]), 1);
        assert_eq!(Rc::strong_count(&pop[1]), 1);
    }

    #[test]
    fn test_interior_mutability_using_scoped_blocks() {
        let mut pop: Vec<IndividualPointer> = vec![];

        pop.push(IndividualPointer::new(RefCell::<Individual>::new(
            Individual::new(0),
        )));
        pop.push(IndividualPointer::new(RefCell::<Individual>::new(
            Individual::new(1),
        )));

        {
            let ind = &mut *pop[0].borrow_mut();

            ind.children.insert(1, vec![Segment::new(0, 0, 2)]);

            let ind = &mut *pop[1].borrow_mut();

            ind.parents.push(pop[0].clone());
        }

        assert_eq!(Rc::strong_count(&pop[0]), 2);
        assert_eq!(Rc::strong_count(&pop[1]), 1);

        remove_parent(pop[0].clone(), pop[1].clone());
        assert_eq!(Rc::strong_count(&pop[0]), 1);
        assert_eq!(Rc::strong_count(&pop[1]), 1);
    }
}

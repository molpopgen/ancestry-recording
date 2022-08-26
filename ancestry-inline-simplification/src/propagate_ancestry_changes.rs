use crate::node::Node;
use crate::node_heap::NodeHeap;
use crate::segments::AncestrySegment;
use crate::segments::HalfOpenInterval;
use crate::InlineAncestryError;

pub fn propagate_ancestry_changes(
    genome_length: crate::LargeSignedInteger,
    node_heap: &mut NodeHeap,
) -> Result<i32, InlineAncestryError> {
    let mut popped = 0;
    while let Some(mut n) = node_heap.pop() {
        popped += 1;
        n.preprocess(genome_length);
        let mut node = Node::from(n);
        #[cfg(debug_assertions)]
        let ancestry = {
            let mut ancestry = vec![];
            for i in node.borrow().ancestry.iter() {
                ancestry.push(i.clone());
            }
            ancestry
        };

        let changed = node.update_ancestry()?;

        #[cfg(debug_assertions)]
        {
            if changed {
                //println!("node {}, {}", node.borrow().index, node.is_alive());
                //println!("{:?}", ancestry);
                //println!("{} children before", nc);
                //println!("+++++");
                //println!("{:?}", node.borrow().ancestry);
                //println!("{} children after", node.borrow().children.len());
                //println!("-----");
                let mut before = vec![];
                if !ancestry.is_empty() {
                    before.push(ancestry[0].clone());
                }
                for a in ancestry.windows(2) {
                    //println!("window {:?}", a);
                    if a[0].child != a[1].child {
                        before.push(a[1].clone());
                    } else {
                        if a[0].segment.right == a[1].segment.left {
                            match before.last_mut() {
                                Some(value) => value.segment.right = a[1].segment.left,
                                None => (),
                            }
                        } else {
                            before.push(a[1].clone());
                        }
                    }
                }
                let mut after = vec![];
                //println!("bsquashed {:?}", before);
                let b = node.borrow();
                if !b.ancestry.is_empty() {
                    after.push(b.ancestry[0].clone());
                }
                for a in b.ancestry.windows(2) {
                    //println!("window {:?}", a);
                    if a[0].child != a[1].child {
                        after.push(a[1].clone());
                    } else {
                        if a[0].segment.right == a[1].segment.left {
                            match after.last_mut() {
                                Some(value) => value.segment.right = a[1].segment.left,
                                None => (),
                            }
                        } else {
                            after.push(a[1].clone());
                        }
                    }
                }
                //println!("asquashed {:?}", after);

                if before == after && !before.is_empty() {
                    assert!(false, "{} {}", changed, b.is_alive());
                }

                //println!("///////");
            }
        }

        if changed {
            //|| node.is_alive() {
            for parent in node.borrow_mut().parents.iter() {
                node_heap.push_parent(parent.clone());
            }
        }
    }
    assert!(node_heap.is_empty());
    Ok(popped)
}

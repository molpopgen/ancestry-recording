pub struct IndexedPopulation {
    nodes: Vec<crate::indexed_node::Node>,
    counts: Vec<i32>,
    // FIFO queue to recycle indexes of extinct (zero) counts
    queue: Vec<usize>,
}

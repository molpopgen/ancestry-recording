use bitflags::bitflags;

pub type SignedInteger = i32;

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub(crate) SignedInteger);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Position(pub(crate) SignedInteger);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Time(pub(crate) SignedInteger);

#[derive(Default)]
pub struct NodeTable {
    flags: Vec<NodeFlags>,
    time: Vec<Time>,
    index: usize,
}

impl NodeTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn row(&self, index: usize) -> (NodeFlags, Time) {
        (self.flags[index], self.time[index])
    }

    pub fn add_row(&mut self, flags: NodeFlags, time: Time) {
        self.flags.push(flags);
        self.time.push(time);
    }
}

impl Iterator for NodeTable {
    type Item = (NodeFlags, Time);

    fn next(&mut self) -> Option<Self::Item> {
        assert_eq!(self.flags.len(), self.time.len());
        if self.index < self.flags.len() {
            let rv = (self.flags[self.index], self.time[self.index]);
            self.index += 1;
            Some(rv)
        } else {
            self.index = 0;
            None
        }
    }
}

bitflags! {
    #[derive(Default)]
    pub struct NodeFlags: u32 {
        const ISALIVE = 1 << 1;
        const ISREMEMBERED = 1 << 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_node_flags() {
        let f = NodeFlags::default();
        assert_eq!(f.bits(), 0);
    }

    #[test]
    fn test_default_index_node_table() {
        let nt = NodeTable::new();
        assert_eq!(nt.index, 0);
    }
}

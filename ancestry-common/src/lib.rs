use bitflags::bitflags;

pub type SignedInteger = i32;

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Node(pub(crate) SignedInteger);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Position(pub(crate) SignedInteger);

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
}

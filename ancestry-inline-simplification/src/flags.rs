use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    pub struct NodeFlags: u32 {
        const IS_ALIVE = 1 << 0;
    }
}

impl NodeFlags {
    pub fn new_alive() -> Self {
        NodeFlags::IS_ALIVE
    }

    pub fn is_alive(&self) -> bool {
        self.contains(NodeFlags::IS_ALIVE)
    }

    pub fn clear_alive(&mut self) {
        self.remove(NodeFlags::IS_ALIVE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let n = NodeFlags::default();
        assert_eq!(n.bits(), 0);
    }

    #[test]
    fn test_alive() {
        let mut n = NodeFlags::new_alive();
        assert!(n.bits() > 0);
        assert!(n.contains(NodeFlags::IS_ALIVE));
        assert!(n.is_alive());
        n.clear_alive();
        assert!(!n.contains(NodeFlags::IS_ALIVE));
        assert!(!n.is_alive());
    }
}

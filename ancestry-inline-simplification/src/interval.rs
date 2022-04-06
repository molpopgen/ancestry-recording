use crate::LargeSignedInteger;

#[derive(Copy, Clone)]
pub struct Interval {
    pub left: LargeSignedInteger,
    pub right: LargeSignedInteger,
}

impl Interval {
    pub fn new(left: LargeSignedInteger, right: LargeSignedInteger) -> Self {
        Self { left, right }
    }
}

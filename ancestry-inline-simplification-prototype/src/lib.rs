pub use ancestry_common::{LargeSignedInteger, Segment, SignedInteger};

pub mod individual;
pub mod simulate;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

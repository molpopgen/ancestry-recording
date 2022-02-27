pub use ancestry_common::{LargeSignedInteger, SignedInteger};

pub mod individual;
pub mod population;
pub mod simulate;
pub mod segment;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

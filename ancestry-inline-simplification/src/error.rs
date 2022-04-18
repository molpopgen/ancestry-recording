use thiserror::Error;

#[derive(Error, Debug)]
pub enum InlineAncestryError {
    #[error("generic placeholder")]
    GenericPlaceholder,
}

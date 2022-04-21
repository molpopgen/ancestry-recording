use crate::HalfOpenInterval;
use crate::InlineAncestryError;

pub(crate) fn non_overlapping_segments<T: HalfOpenInterval>(
    segments: &[T],
) -> Result<(), InlineAncestryError> {
    let sorted = segments.windows(2).all(|w| w[0].left() < w[1].left());

    if sorted {
        Ok(())
    } else {
        Err(InlineAncestryError::IntervalsError)
    }
}

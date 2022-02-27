use crate::LargeSignedInteger;
use crate::SignedInteger;
use rgsl::rng;

fn ran_flat(rng: &mut rng::Rng, lo: f64, hi: f64) -> f64 {
    let mut rv = rng.flat(lo, hi);

    while match rv.partial_cmp(&hi) {
        Some(std::cmp::Ordering::Equal) => true,
        Some(_) => false,
        None => panic!("gsl_rng_ran_flat should not return NaN"),
    } {
        rv = rng.flat(lo, hi);
    }
    rv
}

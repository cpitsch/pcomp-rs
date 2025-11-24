use super::Binner;

/// A simple binner based on Percentiles: The lower and upper x-th percentiles
/// form the 0th and 2nd bin, respectively. The middle values form the 1st bin.
#[derive(Debug)]
pub struct OuterPercentileBinner {
    lower_boundary: f64,
    upper_boundary: f64,
}

impl Binner<f64> for OuterPercentileBinner {
    type Args = f64;

    fn new(mut data: Vec<f64>, args: Self::Args) -> Self {
        // TODO: Could avoid sorting data multiple times by expecting sorted data
        // in `percentile`
        let lower_boundary = percentile(&mut data, args);
        let upper_boundary = percentile(&mut data, 100.0 - args);

        Self {
            lower_boundary,
            upper_boundary,
        }
    }
    fn num_bins(&self) -> usize {
        3
    }

    fn bin(&self, data: f64) -> usize {
        if data < self.lower_boundary {
            0
        } else if data < self.upper_boundary {
            1
        } else {
            2
        }
    }
}

/// Get the x-th percentile of the data.
///
/// `percentile` is expected to be in the range [0.0, 100.0]. If this is not the
/// case, the function panics.
fn percentile(data: &mut [f64], percentile: f64) -> f64 {
    if !(0.0..=100.0).contains(&percentile) {
        panic!("Invalid percentile.")
    }

    data.sort_by(|a, b| a.total_cmp(b));

    let rank = percentile / 100.0 * (data.len() - 1) as f64;
    let lower_index = rank.floor() as usize;
    let upper_index = rank.ceil() as usize;

    if lower_index == upper_index {
        data[lower_index]
    } else {
        let lower_value = data[lower_index];
        let upper_value = data[upper_index];
        lower_value + (upper_value - lower_value) * (rank - lower_index as f64)
    }
}

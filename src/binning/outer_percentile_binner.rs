use super::Binner;

pub struct OuterPercentileBinner {
    lower_boundary: f64,
    upper_boundary: f64,
}

impl Binner<f64> for OuterPercentileBinner {
    type Args = ();

    fn new(mut data: Vec<f64>, _args: Self::Args) -> Self {
        let lower_boundary = percentile(&mut data, 10.0);
        let upper_boundary = percentile(&mut data, 90.0);

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

fn percentile(data: &mut [f64], percentile: f64) -> f64 {
    if !(0.0..=100.0).contains(&percentile) {
        panic!("Invalid percentile.")
    }

    data.sort_by(|a, b| a.partial_cmp(b).unwrap());

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

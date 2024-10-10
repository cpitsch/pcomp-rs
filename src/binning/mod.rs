use std::{collections::HashMap, usize};

pub mod kmeans_binner;
pub mod outer_percentile_binner;

pub trait Binner<U> {
    type Args: Clone;

    fn new(data: Vec<U>, args: Self::Args) -> Self;
    fn bin(&self, data: U) -> usize;
    fn num_bins(&self) -> usize;
}

pub struct BinnerManager<U, T: Binner<U>> {
    binners: HashMap<String, T>,
    _phantom: Option<U>,
}

impl<U, T> BinnerManager<U, T>
where
    T: Binner<U>,
{
    pub fn bin(&self, label: &String, data: U) -> usize {
        self.binners.get(label).unwrap().bin(data)
    }

    pub fn from_key_value_pairs(data: Vec<(String, U)>, binner_args: T::Args) -> Self {
        let mut grouped_data: HashMap<String, Vec<U>> = HashMap::new();
        data.into_iter().for_each(|(k, v)| {
            grouped_data.entry(k).or_default().push(v);
        });

        let binners: HashMap<String, T> = grouped_data
            .into_iter()
            .map(|(k, v)| (k, T::new(v, binner_args.clone())))
            .collect();

        BinnerManager {
            binners,
            _phantom: None,
        }
    }
}

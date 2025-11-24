use std::{collections::HashMap, marker::PhantomData};

pub mod kmeans_binner;
pub mod outer_percentile_binner;

pub trait Binner<U> {
    type Args: Clone;

    fn new(data: Vec<U>, args: Self::Args) -> Self;
    fn bin(&self, data: U) -> usize;
    fn num_bins(&self) -> usize;
}

/// Train and manage a separate binner for each "key" (activity).
#[derive(Debug)]
pub struct BinnerManager<U, T: Binner<U>> {
    binners: HashMap<String, T>,

    // `U` (the data type of the unbinned values) needs to be used inside the binner
    // manager. `PhantomData` does this for us.
    #[doc(hidden)]
    _phantom: PhantomData<U>,
}

impl<U, T> BinnerManager<U, T>
where
    T: Binner<U>,
{
    /// Bin a value for a certain class (activity).
    ///
    /// Panics if the activity was not in the training data.
    pub fn bin(&self, label: &str, data: U) -> usize {
        self.binners.get(label).unwrap().bin(data)
    }

    /// Create a [`BinnerManager`] from (key, value) pairs. For each unique key, a
    /// binner is created trained on the respective values.
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
            _phantom: PhantomData,
        }
    }
}

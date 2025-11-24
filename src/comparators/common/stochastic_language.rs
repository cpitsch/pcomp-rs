use std::hash::Hash;

use itertools::{multiunzip, Itertools};
use ndarray::Array1;

/// Tracks relative frequencies of items (traces).
#[derive(Debug)]
pub struct StochasticLanguage<T: Hash + Eq + Clone> {
    pub variants: Vec<T>,
    pub frequencies: Array1<f64>,
}

impl<T> StochasticLanguage<T>
where
    T: Hash + Eq + Clone + PartialOrd,
{
    /// Create a [`StochasticLanguage`] from a `Vec` of items their relative frequency.
    pub fn from_vec(v: Vec<(T, f64)>) -> Self {
        let v_len = v.len();
        let mut variants: Vec<T> = Vec::with_capacity(v_len);
        let mut frequencies = Array1::zeros(v_len);

        v.into_iter().enumerate().for_each(|(i, (variant, freq))| {
            variants.push(variant);
            frequencies[i] = freq;
        });

        Self {
            variants,
            frequencies,
        }
    }

    /// Iterate over `(item, relative_frequency)` pairs.
    pub fn iter_pairs(
        &self,
    ) -> std::iter::Zip<
        std::slice::Iter<'_, T>,
        ndarray::iter::Iter<'_, f64, ndarray::Dim<[usize; 1]>>,
    > {
        let res = self.variants.iter().zip(self.frequencies.iter());
        res
    }

    /// Create a [`StochasticLanguage`] from a `Vec` of items
    pub fn from_items(items: Vec<T>) -> Self {
        let population_size: f64 = items.len() as f64;
        items
            .into_iter()
            .counts()
            .into_iter()
            .map(|(k, v)| (k, v as f64 / population_size))
            .sorted_by(|x, y| x.partial_cmp(y).unwrap())
            .collect()
    }
}

impl<T> FromIterator<(T, f64)> for StochasticLanguage<T>
where
    T: Hash + Eq + Clone,
{
    fn from_iter<I: IntoIterator<Item = (T, f64)>>(iter: I) -> Self {
        let (variants, frequencies) = multiunzip(iter);
        let frequencies = Array1::from_vec(frequencies);
        Self {
            variants,
            frequencies,
        }
    }
}

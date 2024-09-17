use std::hash::Hash;

use itertools::{multiunzip, Itertools};

pub struct StochasticLanguage<T: Hash + Eq + Clone> {
    pub variants: Vec<T>,
    pub frequencies: Vec<f64>,
}

impl<T> StochasticLanguage<T>
where
    T: Hash + Eq + Clone + PartialOrd,
{
    pub fn from_vec(v: Vec<(T, f64)>) -> Self {
        v.into_iter().collect()
    }

    pub fn iter_pairs(&self) -> std::iter::Zip<std::slice::Iter<'_, T>, std::slice::Iter<'_, f64>> {
        self.variants.iter().zip(self.frequencies.iter())
    }

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
        Self {
            variants,
            frequencies,
        }
    }
}

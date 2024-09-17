use std::{collections::HashSet, fmt::Debug, hash::Hash};

use itertools::Itertools;
use ndarray::Array2;
use process_mining::EventLog;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};

use crate::{
    comparators::common::stochastic_language::StochasticLanguage, emd::compute_emd,
    utils::progress::build_progress_bar,
};

#[derive(Debug)]
pub struct PermutationTestComparisonResult {
    pub logs_emd: f64,
    pub permutation_emds: Vec<f64>,
    pub pvalue: f64,
}

pub trait PermutationTestComparator<T>
where
    T: Hash + Eq + Clone + Ord + Debug,
{
    // fn extract_representation(&self, trace: &Trace) -> T;
    fn cost(&self, rep_1: &T, rep_2: &T) -> f64;

    fn extract_representations(&self, log_1: &EventLog, log_2: &EventLog) -> (Vec<T>, Vec<T>);

    fn compare(
        &self,
        log_1: &EventLog,
        log_2: &EventLog,
        distribution_size: usize,
    ) -> PermutationTestComparisonResult {
        let (behavior_1, behavior_2) = self.extract_representations(log_1, log_2);

        let mut combined_variants: Vec<T> = behavior_1 // Use a Vec so the order is fixed
            .iter()
            .chain(behavior_2.iter())
            .cloned()
            .collect::<HashSet<T>>()
            .into_iter()
            .collect();
        combined_variants.sort();
        let stoch_lang_1 = StochasticLanguage::from_items(behavior_1.clone());
        let stoch_lang_2 = StochasticLanguage::from_items(behavior_2.clone());

        let large_distance_matrix = self.compute_symmetric_distance_matrix(&combined_variants);

        let log_1_log_2_distances = project_distance_matrix(
            &large_distance_matrix,
            &combined_variants,
            &stoch_lang_1,
            &stoch_lang_2,
        );

        let logs_emd = compute_emd(
            stoch_lang_1.frequencies.clone(),
            stoch_lang_2.frequencies.clone(),
            &log_1_log_2_distances,
        )
        .emd;

        let permutation_emds = compute_permutation_test_distribution(
            &large_distance_matrix,
            combined_variants,
            behavior_1,
            behavior_2,
            distribution_size,
        );

        let pvalue = permutation_emds
            .iter()
            .filter(|emd| **emd > logs_emd)
            .collect_vec()
            .len() as f64
            / distribution_size as f64;

        PermutationTestComparisonResult {
            logs_emd,
            pvalue,
            permutation_emds,
        }
    }

    fn compute_symmetric_distance_matrix(&self, variants: &[T]) -> Array2<f64> {
        let mut mat = Array2::zeros((variants.len(), variants.len()));
        let progress = build_progress_bar(
            variants.len().pow(2) as u64,
            format!(
                "Computing complete distance matrix ({}x{})",
                mat.shape()[0],
                mat.shape()[1]
            ),
        );

        variants.iter().enumerate().for_each(|(i, item_1)| {
            variants.iter().enumerate().skip(i).for_each(|(j, item_2)| {
                mat[(i, j)] = self.cost(item_1, item_2);
                mat[(j, i)] = mat[(i, j)];

                progress.inc(if i != j { 2 } else { 1 });
            })
        });

        progress.finish();

        mat

        // Would be much more elegant, if only there was an implementation for
        // symmetric matrices (skips the other half)
        // Array2::from_shape_fn(|i, j| self.cost(variants[i], variants[j]))
    }
}

pub fn project_distance_matrix<T: Clone + Eq + Hash>(
    dists: &Array2<f64>,
    dist_matrix_source_population: &[T],
    population_1: &StochasticLanguage<T>,
    population_2: &StochasticLanguage<T>,
) -> Array2<f64> {
    let pop_1_indices: Vec<usize> = population_1
        .variants
        .iter()
        .map(|item| {
            dist_matrix_source_population
                .iter()
                .position(|x| x == item)
                .unwrap()
        })
        .collect();
    let pop_2_indices: Vec<usize> = population_2
        .variants
        .iter()
        .map(|item| {
            dist_matrix_source_population
                .iter()
                .position(|x| x == item)
                .unwrap()
        })
        .collect();

    let selected_rows = dists.select(ndarray::Axis(0), &pop_1_indices);
    selected_rows.select(ndarray::Axis(1), &pop_2_indices)
}

pub fn compute_permutation_test_distribution<T: PartialEq>(
    dists: &Array2<f64>,
    distance_matrix_source_population: Vec<T>,
    behavior_1: Vec<T>,
    behavior_2: Vec<T>,
    distribution_size: usize,
) -> Vec<f64> {
    let population_indices_to_variant_indices: Vec<usize> = behavior_1
        .iter()
        .chain(behavior_2.iter())
        .map(|item| {
            distance_matrix_source_population
                .iter()
                .position(|x| x == item)
                .unwrap()
        })
        .collect();
    let sample_size = behavior_1.len() + behavior_2.len();

    let mut rng = StdRng::from_entropy();

    let progress = build_progress_bar(
        distribution_size as u64,
        "Computing permutation EMD distribution".into(),
    );
    let res = (0..distribution_size)
        .map(|_| {
            let mut sample = (0..sample_size).collect_vec();
            sample.partial_shuffle(&mut rng, behavior_1.len());
            let (sample_1, sample_2) = sample.split_at(behavior_1.len());
            let translated_sample_1: StochasticLanguage<usize> = sample_1
                .iter()
                .map(|index| population_indices_to_variant_indices[*index])
                .counts()
                .into_iter()
                .map(|(k, v)| (k, v as f64 / behavior_1.len() as f64))
                .collect();
            let translated_sample_2: StochasticLanguage<usize> = sample_2
                .iter()
                .map(|index| population_indices_to_variant_indices[*index])
                .counts()
                .into_iter()
                .map(|(k, v)| (k, v as f64 / behavior_1.len() as f64))
                .collect();

            let projected_dists = dists
                .select(ndarray::Axis(0), &translated_sample_1.variants)
                .select(ndarray::Axis(1), &translated_sample_2.variants);

            let res = compute_emd(
                translated_sample_1.frequencies,
                translated_sample_2.frequencies,
                &projected_dists,
            )
            .emd;
            progress.inc(1);
            res
        })
        .collect();
    progress.finish();
    res
}

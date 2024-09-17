use std::{fmt::Debug, hash::Hash};

use itertools::Itertools;
use ndarray::Array2;
use process_mining::EventLog;
use rand::{
    distributions::{Distribution, WeightedIndex},
    rngs::StdRng,
    SeedableRng,
};

use crate::{
    comparators::common::stochastic_language::StochasticLanguage, emd::compute_emd,
    utils::progress::build_progress_bar,
};

#[derive(Debug)]
pub struct BootstrapTestComparisonResult {
    pub logs_emd: f64,
    pub bootstrap_emds: Vec<f64>,
    pub pvalue: f64,
}

pub trait BootstrapTestComparator<T>
where
    T: Hash + Eq + Clone + Ord + Debug,
{
    fn cost(&self, rep_1: &T, rep_2: &T) -> f64;

    fn extract_representations(&self, log_1: &EventLog, log_2: &EventLog) -> (Vec<T>, Vec<T>);

    fn compare(
        &self,
        log_1: &EventLog,
        log_2: &EventLog,
        resample_size: usize,
        distribution_size: usize,
        seed: Option<u64>,
    ) -> BootstrapTestComparisonResult {
        let (behavior_1, behavior_2) = self.extract_representations(log_1, log_2);

        let stoch_lang_1 = StochasticLanguage::from_items(behavior_1);
        let stoch_lang_2 = StochasticLanguage::from_items(behavior_2);

        let logs_emd = compute_emd(
            stoch_lang_1.frequencies.clone(),
            stoch_lang_2.frequencies.clone(),
            &self.compute_distance_matrix(&stoch_lang_1.variants, &stoch_lang_2.variants),
        )
        .emd;

        let bootstrap_emds =
            self.bootstrap_emd_population(stoch_lang_1, resample_size, distribution_size, seed);

        let pvalue = bootstrap_emds
            .iter()
            .filter(|emd| **emd > logs_emd)
            .collect_vec()
            .len() as f64
            / distribution_size as f64;

        BootstrapTestComparisonResult {
            logs_emd,
            bootstrap_emds,
            pvalue,
        }
    }

    fn compute_distance_matrix(&self, variants_1: &[T], variants_2: &[T]) -> Array2<f64> {
        let progress = build_progress_bar(
            variants_1.len() as u64 * variants_2.len() as u64,
            format!(
                "Computing complete distance matrix ({}x{})",
                variants_1.len(),
                variants_2.len(),
            ),
        );

        let dists = Array2::from_shape_fn((variants_1.len(), variants_2.len()), |(i, j)| {
            let res = self.cost(&variants_1[i], &variants_2[j]);
            progress.inc(1);
            res
        });
        progress.finish();
        dists
    }

    fn bootstrap_emd_population(
        &self,
        reference_stochastic_language: StochasticLanguage<T>,
        resample_size: usize,
        distribution_size: usize,
        seed: Option<u64>,
    ) -> Vec<f64> {
        let distance_matrix = self.compute_distance_matrix(
            &reference_stochastic_language.variants,
            &reference_stochastic_language.variants,
        );

        let mut sampler = WeightedIndex::new(reference_stochastic_language.frequencies.clone())
            .unwrap()
            .sample_iter(if let Some(s) = seed {
                StdRng::seed_from_u64(s)
            } else {
                StdRng::from_entropy()
            });

        let progress = build_progress_bar(
            distribution_size as u64,
            "Computing permutation EMD distribution".into(),
        );

        let emds = (0..distribution_size)
            .map(|_| {
                let sample_indices: Vec<usize> = sampler.by_ref().take(resample_size).collect();
                let sample_stochastic_language = StochasticLanguage::from_items(sample_indices);
                let projected_costs =
                    distance_matrix.select(ndarray::Axis(0), &sample_stochastic_language.variants);
                let emd = compute_emd(
                    sample_stochastic_language.frequencies,
                    reference_stochastic_language.frequencies.clone(),
                    &projected_costs,
                )
                .emd;
                progress.inc(1);
                emd
            })
            .collect();
        progress.finish();
        emds
    }
}

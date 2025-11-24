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
    comparators::common::stochastic_language::StochasticLanguage,
    emd::compute_emd,
    utils::{attributes::attribute_error::AttributeResult, progress::build_progress_bar},
};

#[derive(Debug)]
pub struct BootstrapTestComparisonResult {
    /// The EMD measured between the two event logs.
    pub logs_emd: f64,
    /// The EMDs computed in the bootstrapping phase.
    pub bootstrap_emds: Vec<f64>,
    /// The computed p-value
    pub pvalue: f64,
}

/// The Bootstrap Method for Process Hypothesis Testing proposed in "Statistical
/// tests and association measures for business processes" by Leemans et al. (The
/// P-P-UP test).
///
/// A bootstrap distribution is created by repeatedly computing the EMD between
/// the first event log and a sample of itself (with replacement). Then, the p-value
/// is computed as the fraction of bootstrap distribution values greater than the
/// EMD between the two event logs.
pub trait BootstrapTestComparator<T>
where
    T: Hash + Eq + Clone + Ord + Debug,
{
    /// The cost (dissimilarity) function between two representations.
    fn cost(&self, rep_1: &T, rep_2: &T) -> f64;

    /// Map each case to a _representation_, capturing the information relevant
    /// to the comparison. Can also include preprocessing, e.g., binning of continuous
    /// values.
    fn extract_representations(
        &self,
        log_1: &EventLog,
        log_2: &EventLog,
    ) -> AttributeResult<(Vec<T>, Vec<T>)>;

    /// Compare two event logs.
    ///
    /// - Returns an `Err` if required attributes are not present on the events.
    ///     - For a control-flow comparison, this is the activity label `concept:name`
    ///     - For timed control flow, this is additionally the start and completion timestamps
    ///       `start_timestamp` and `time:timestamp`.
    ///       - In case you are using an event log without `start_timestamp`, see
    ///         [`ensure_start_timestamp_key`]
    ///
    /// [`ensure_start_timestamp_key`]: crate::comparators::common::preparation::ensure_start_timestamp_key
    fn compare(
        &self,
        log_1: &EventLog,
        log_2: &EventLog,
        resample_size: usize,
        distribution_size: usize,
        seed: Option<u64>,
    ) -> AttributeResult<BootstrapTestComparisonResult> {
        let (behavior_1, behavior_2) = self.extract_representations(log_1, log_2)?;

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

        Ok(BootstrapTestComparisonResult {
            logs_emd,
            bootstrap_emds,
            pvalue,
        })
    }

    /// Compute the distance matrix between two collections of variants using
    /// the [`cost`] function.
    ///
    /// The output matrix has dimensions `(variants_1.len(), variants_2.len())`.
    ///
    /// [`cost`]: BootstrapTestComparator::cost
    fn compute_distance_matrix(&self, variants_1: &[T], variants_2: &[T]) -> Array2<f64> {
        let progress = build_progress_bar(
            variants_1.len() as u64 * variants_2.len() as u64,
            format!(
                "Computing distance matrix ({}x{})",
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

    /// Compute the bootstrap distribution by repeatedly taking samples of size
    /// `resample_size` from `reference_stochastic_language` with replacement,
    /// and computing the EMD to `reference_stochastic_language`.
    ///
    /// * `reference_stochastic_language`: The stochastic language of the event
    ///   log considered for the bootstrap method.
    /// * `resample_size`: The size of the samples in the bootstrap method.
    /// * `distribution_size`: The number of repititions (the size of the resulting
    ///   bootstrap distribution).
    /// * `seed`: An (optional) seed to use for sampling.
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

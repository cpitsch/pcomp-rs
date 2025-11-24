use just_emd::{EmdResult, EmdSolver};
use ndarray::{Array1, Array2};

/// Compute the Earth Mover's Distance (EMD) between two populations given as an
/// array of relative frequencies.
pub fn compute_emd(
    mut frequencies_1: Array1<f64>,
    mut frequencies_2: Array1<f64>,
    distances: &Array2<f64>,
) -> EmdResult {
    EmdSolver::new(
        &mut frequencies_1,
        &mut frequencies_2,
        &mut distances.as_standard_layout().to_owned(),
    )
    .solve()
    // By construction of the EMD (Same capacity on both sides, fully connected bipartite
    // graph, ..), there should always be a solution.
    // WARN: Unless max iter is reached?
    .unwrap()
}

use ndarray::{Array1, Array2};
use network_simplex::{EmdResult, EmdSolver};

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
    .unwrap()
}

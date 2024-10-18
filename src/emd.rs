use ndarray::{Array1, Array2};
use network_simplex::{EmdResult, EmdSolver};

pub fn compute_emd(
    frequencies_1: Vec<f64>,
    frequencies_2: Vec<f64>,
    distances: &Array2<f64>,
) -> EmdResult {
    EmdSolver::new(
        &mut Array1::from_iter(frequencies_1),
        &mut Array1::from_iter(frequencies_2),
        &mut distances.as_standard_layout().to_owned(),
    )
    .solve()
    .unwrap()
}

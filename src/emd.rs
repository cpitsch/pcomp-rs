use ndarray::{Array1, Array2};
use rust_optimal_transport::{exact::EarthMovers, OTSolver};

pub struct EmdResult {
    pub flow_matrix: Array2<f64>,
    pub emd: f64,
}

pub fn compute_emd(
    frequencies_1: Vec<f64>,
    frequencies_2: Vec<f64>,
    distances: &Array2<f64>,
) -> EmdResult {
    let ot_matrix = EarthMovers::new(
        &mut Array1::from_vec(frequencies_1.clone()),
        &mut Array1::from_vec(frequencies_2.clone()),
        &mut distances.as_standard_layout().to_owned(),
    )
    .solve()
    .unwrap();

    EmdResult {
        emd: (&ot_matrix * distances).sum(),
        flow_matrix: ot_matrix,
    }
}

// pub fn scip_lp(frequencies_1: Vec<f64>, frequencies_2: Vec<f64>, distances: &Array2<f64>) {
//     let mut model = Model::new()
//         .hide_output()
//         .include_default_plugins()
//         .create_prob("EMD")
//         .set_obj_sense(ObjSense::Minimize);
//     let mut edge_vars: Vec<Vec<Rc<Variable>>> = vec![vec![]; frequencies_1.len()];
//
//     for (i, f1) in frequencies_1.iter().enumerate() {
//         for (j, _f2) in frequencies_2.iter().enumerate() {
//             edge_vars[i].push(model.add_var(
//                 0.0,
//                 f64::INFINITY,
//                 distances[(i, j)],
//                 format!("({i},{j})").as_str(),
//                 russcip::VarType::Continuous,
//             ));
//         }
//         model.add_cons(
//             edge_vars[i].clone(),
//             &vec![1.0; frequencies_2.len()],
//             f64::INFINITY,
//             *f1,
//             format!("i{i}_supply").as_str(),
//         );
//     }
//
//     frequencies_2.iter().enumerate().for_each(|(j, f2)| {
//         let f2_vars = (0..frequencies_1.len())
//             .map(|i| edge_vars[i][j].clone())
//             .collect();
//         model.add_cons(
//             f2_vars,
//             &vec![1.0; frequencies_1.len()],
//             f64::INFINITY,
//             *f2,
//             format!("j{j}_supply").as_str(),
//         );
//     });
//
//     let sum_1: f64 = frequencies_1.iter().sum();
//     let sum_2: f64 = frequencies_2.iter().sum();
//     model.add_cons(
//         edge_vars.iter().flatten().cloned().collect(),
//         &vec![1.0; frequencies_1.len() * frequencies_2.len()],
//         0.0,
//         f64_min(sum_1, sum_2),
//         "total_flow_constraint",
//     );
//     println!("Ready to solve!");
//
//     let model = model.solve();
//
//     println!("Objective: {}", model.obj_val());
// }

// pub fn emd_lp_2(frequencies_1: Vec<f64>, frequencies_2: Vec<f64>, distances: &Array2<f64>) {
//     // Skips nodes with frequency 0
//     let mut problem = Problem::new(OptimizationDirection::Minimize);
//     let mut edge_vars: HashMap<(usize, usize), minilp::Variable> = HashMap::new();
//
//     frequencies_1
//         .iter()
//         .enumerate()
//         .filter(|(_i, x)| **x > 0.0)
//         .for_each(|(i, _supply)| {
//             frequencies_2
//                 .iter()
//                 .enumerate()
//                 .filter(|(_j, y)| **y > 0.0)
//                 .for_each(|(j, _demand)| {
//                     edge_vars.insert((i, j), problem.add_var(distances[(i, j)], (0.0, 1.0)));
//                 })
//         });
//
//     frequencies_1
//         .iter()
//         .enumerate()
//         .filter(|(_i, x)| **x > 0.0)
//         .for_each(|(i, supply)| {
//             let mut sum = LinearExpr::empty();
//             frequencies_2
//                 .iter()
//                 .enumerate()
//                 .filter(|(_j, y)| **y > 0.0)
//                 .for_each(|(j, _demand)| {
//                     sum.add(edge_vars[&(i, j)], 1.0);
//                 });
//             problem.add_constraint(sum, ComparisonOp::Le, *supply);
//         });
//
//     frequencies_2
//         .iter()
//         .enumerate()
//         .filter(|(_j, y)| **y > 0.0)
//         .for_each(|(j, demand)| {
//             let mut sum = LinearExpr::empty();
//             frequencies_1
//                 .iter()
//                 .enumerate()
//                 .filter(|(_i, x)| **x > 0.0)
//                 .for_each(|(i, _supply)| {
//                     sum.add(edge_vars[&(i, j)], 1.0);
//                 });
//             problem.add_constraint(sum, ComparisonOp::Le, *demand);
//         });
//
//     let mut total_sum = LinearExpr::empty();
//     edge_vars
//         .iter()
//         .for_each(|(_k, var)| total_sum.add(*var, 1.0));
//
//     let sum_1: f64 = frequencies_1.iter().sum();
//     let sum_2: f64 = frequencies_2.iter().sum();
//     problem.add_constraint(total_sum, ComparisonOp::Eq, f64_min(sum_1, sum_2));
//
//     println!("Ready to solve!");
//     let solution = problem.solve().unwrap();
//     println!("Objective: {}", solution.objective());
// }
//
// pub fn emd_lp(frequencies_1: Vec<f64>, frequencies_2: Vec<f64>, distances: &Array2<f64>) {
//     let mut problem = Problem::new(OptimizationDirection::Maximize);
//     let mut edge_vars: Vec<Vec<minilp::Variable>> = vec![vec![]; frequencies_2.len()];
//
//     for (i, f1) in frequencies_1.iter().enumerate() {
//         let mut f1_sum = LinearExpr::empty();
//         for (j, _f2) in frequencies_2.iter().enumerate() {
//             let edge_var = problem.add_var(distances[(i, j)], (0.0, 1.0));
//             f1_sum.add(edge_var, 1.0);
//             edge_vars[j].push(edge_var);
//         }
//         problem.add_constraint(f1_sum, ComparisonOp::Le, *f1);
//     }
//
//     frequencies_2.iter().enumerate().for_each(|(j, f2)| {
//         let mut f2_sum = LinearExpr::empty();
//         edge_vars[j]
//             .iter()
//             .for_each(|edge_var| f2_sum.add(*edge_var, 1.0));
//         problem.add_constraint(f2_sum, ComparisonOp::Le, *f2)
//     });
//
//     let mut total_sum = LinearExpr::empty();
//     edge_vars.iter().flatten().for_each(|edge_var| {
//         total_sum.add(*edge_var, 1.0);
//     });
//     let sum_1: f64 = frequencies_1.iter().sum();
//     let sum_2: f64 = frequencies_2.iter().sum();
//
//     problem.add_constraint(total_sum, ComparisonOp::Eq, f64_min(sum_1, sum_2));
//
//     println!("Ready to solve!");
//     let solution = problem.solve().unwrap();
//     println!("Objective: {}", solution.objective());
// }
//
// fn f64_min(x: f64, y: f64) -> f64 {
//     if x < y {
//         x
//     } else {
//         y
//     }
// }

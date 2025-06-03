use std::cmp::max;

use ndarray::Array2;

pub trait LevenshteinDistance: PartialEq {
    fn insertion_cost(&self) -> f64;
    fn deletion_cost(&self) -> f64;
    fn substitution_cost(&self, other: &Self) -> f64;
}

pub fn weighted_levenshtein_distance<T>(trace_1: &[T], trace_2: &[T]) -> f64
where
    T: LevenshteinDistance,
{
    if trace_1 == trace_2 {
        return 0.0;
    }

    let len_1 = trace_1.len();
    let len_2 = trace_2.len();

    // Zero matrix, first row and column count up since one can simply drop the remaining
    // characters to match the strings if the other string is empty
    let mut matrix: Array2<f64> = Array2::zeros((len_1 + 1, len_2 + 1));
    for i in 1..=len_1 {
        matrix[(i, 0)] = matrix[(i - 1, 0)] + trace_1[i - 1].insertion_cost();
    }
    for j in 1..=len_2 {
        matrix[(0, j)] = matrix[(0, j - 1)] + trace_2[j - 1].insertion_cost();
    }

    trace_1.iter().enumerate().for_each(|(i, event_1)| {
        trace_2.iter().enumerate().for_each(|(j, event_2)| {
            let deletion_cost = event_1.deletion_cost();
            let insertion_cost = event_2.insertion_cost();
            let substitution_cost = event_1.substitution_cost(event_2);
            matrix[(i + 1, j + 1)] = triple_min_f64(
                matrix[(i, j + 1)] + deletion_cost,  // deletion
                matrix[(i + 1, j)] + insertion_cost, // insertion
                matrix[(i, j)] + substitution_cost,  // substitution
            )
        });
    });

    matrix[(len_1, len_2)]
}

pub fn postnormalized_weighted_levenshtein_distance<T>(trace_1: &[T], trace_2: &[T]) -> f64
where
    T: LevenshteinDistance,
{
    let length: f64 = max(trace_1.len(), trace_2.len()) as f64;
    let distance = weighted_levenshtein_distance(trace_1, trace_2);

    distance / length as f64
}

// fn triple_min<T>(x: T, y: T, z: T) -> T
// where
//     T: Ord,
// {
//     min(x, min(y, z))
// }

fn float_min(x: f64, y: f64) -> f64 {
    if x < y {
        x
    } else {
        y
    }
}

fn triple_min_f64(x: f64, y: f64, z: f64) -> f64 {
    float_min(x, float_min(y, z))
}

impl LevenshteinDistance for String {
    fn insertion_cost(&self) -> f64 {
        1.0
    }
    fn deletion_cost(&self) -> f64 {
        1.0
    }
    fn substitution_cost(&self, other: &Self) -> f64 {
        if self == other {
            0.0
        } else {
            1.0
        }
    }
}

impl LevenshteinDistance for char {
    fn insertion_cost(&self) -> f64 {
        1.0
    }
    fn deletion_cost(&self) -> f64 {
        1.0
    }
    fn substitution_cost(&self, other: &Self) -> f64 {
        if self == other {
            0.0
        } else {
            1.0
        }
    }
}

impl LevenshteinDistance for (String, usize) {
    fn insertion_cost(&self) -> f64 {
        0.5 * (1 + self.1) as f64
    }
    fn deletion_cost(&self) -> f64 {
        0.5 * (1 + self.1) as f64
    }
    fn substitution_cost(&self, other: &Self) -> f64 {
        let string_cost = if self.0 == other.0 { 0.0 } else { 1.0 };
        // usize absolute difference
        let usize_cost = if self.1 > other.1 {
            self.1 - other.1
        } else {
            other.1 - self.1
        };

        // Assuming the largest bin is 2, scales from 0 to 1
        let scaled_usize_cost = usize_cost as f64 / 2.0;
        0.5 * (string_cost + scaled_usize_cost)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_wikipedia_example_1() {
        let str_1: Vec<char> = "kitten".chars().collect();
        let str_2: Vec<char> = "sitting".chars().collect();

        assert_eq!(weighted_levenshtein_distance(&str_1, &str_2), 3.0);
        assert_eq!(
            postnormalized_weighted_levenshtein_distance(&str_1, &str_2),
            3.0 / 7.0
        );
    }

    #[test]
    fn test_levenshtein_wikipedia_example_2() {
        let str_1: Vec<char> = "Saturday".chars().collect();
        let str_2: Vec<char> = "Sunday".chars().collect();

        assert_eq!(weighted_levenshtein_distance(&str_1, &str_2), 3.0);
        assert_eq!(
            postnormalized_weighted_levenshtein_distance(&str_1, &str_2),
            3.0 / 8.0
        );
    }

    #[test]
    fn test_postnormalized_weighted_lev_distance() {
        let trace_1: Vec<(String, usize)> = vec![
            ("a".into(), 1),
            ("b".into(), 1),
            ("c".into(), 2),
            ("d".into(), 2),
        ];
        let trace_2: Vec<(String, usize)> = vec![
            ("a".into(), 1),
            ("c".into(), 2),
            ("b".into(), 1),
            ("d".into(), 0),
        ];
        // Solution:
        //   1) Match (a,1) and (a,1) with cost 0
        //   2) Delete (b,1) with cost 0.5 + 0.25 = 0.75
        //   3) Match (c,2) and (c,2) with cost 0
        //   4) Insert a (b,1) with cost 0.5 + 0.25 = 0.75
        //   5) Match (d,2) and (d,0) with cost 0.5 + 0.5 = 2
        // Total cost is 2.0

        assert_eq!(weighted_levenshtein_distance(&trace_1, &trace_2), 2.0);
        assert_eq!(
            postnormalized_weighted_levenshtein_distance(&trace_1, &trace_2),
            2.0 / 4.0
        )
    }
}

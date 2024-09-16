use std::{
    cmp::{max, min},
    usize,
};

use ndarray::Array2;

pub trait LevenshteinDistance: PartialEq {
    fn insertion_cost(&self) -> usize;
    fn deletion_cost(&self) -> usize;
    fn substitution_cost(&self, other: &Self) -> usize;
}

pub fn weighted_levenshtein_distance<T>(trace_1: &[T], trace_2: &[T]) -> usize
where
    T: LevenshteinDistance,
{
    let len_1 = trace_1.len();
    let len_2 = trace_2.len();

    // Zero matrix, first row and column count up since one can simply drop the remaining
    // characters to match the strings if the other string is empty
    let mut matrix: Array2<usize> = Array2::zeros((len_1 + 1, len_2 + 1));
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
            matrix[(i + 1, j + 1)] = triple_min(
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

    distance as f64 / length as f64
}

fn triple_min<T>(x: T, y: T, z: T) -> T
where
    T: Ord,
{
    min(x, min(y, z))
}

impl LevenshteinDistance for String {
    fn insertion_cost(&self) -> usize {
        1
    }
    fn deletion_cost(&self) -> usize {
        1
    }
    fn substitution_cost(&self, other: &Self) -> usize {
        if self == other {
            0
        } else {
            1
        }
    }
}

impl LevenshteinDistance for char {
    fn insertion_cost(&self) -> usize {
        1
    }
    fn deletion_cost(&self) -> usize {
        1
    }
    fn substitution_cost(&self, other: &Self) -> usize {
        if self == other {
            0
        } else {
            1
        }
    }
}

impl LevenshteinDistance for (String, usize) {
    fn insertion_cost(&self) -> usize {
        1 + self.1
    }
    fn deletion_cost(&self) -> usize {
        1 + self.1
    }
    fn substitution_cost(&self, other: &Self) -> usize {
        let string_cost = if self.0 == other.0 { 0 } else { 1 };
        // usize absolute difference
        let usize_cost = if self.1 > other.1 {
            self.1 - other.1
        } else {
            other.1 - self.1
        };
        string_cost + usize_cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_wikipedia_example_1() {
        let str_1: Vec<char> = "kitten".chars().collect();
        let str_2: Vec<char> = "sitting".chars().collect();

        assert_eq!(weighted_levenshtein_distance(&str_1, &str_2), 3);
        assert_eq!(
            postnormalized_weighted_levenshtein_distance(&str_1, &str_2),
            3.0 / 7.0
        );
    }

    #[test]
    fn test_levenshtein_wikipedia_example_2() {
        let str_1: Vec<char> = "Saturday".chars().collect();
        let str_2: Vec<char> = "Sunday".chars().collect();

        assert_eq!(weighted_levenshtein_distance(&str_1, &str_2), 3);
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
            ("d".into(), 3),
        ];
        let trace_2: Vec<(String, usize)> = vec![
            ("a".into(), 1),
            ("c".into(), 2),
            ("b".into(), 1),
            ("d".into(), 1),
        ];
        // Solution:
        //   1) Match (a,1) and (a,1) with cost 0
        //   2) Delete (b,1) with cost 2
        //   3) Match (c,2) and (c,2) with cost 0
        //   4) Insert a (b,1) with cost 2
        //   5) Match (d,3) and (d,1) with cost 2
        // Total cost is 6
        // Alternatively: Match, Rename, Rename, Match => 6

        assert_eq!(weighted_levenshtein_distance(&trace_1, &trace_2), 6);
        assert_eq!(
            postnormalized_weighted_levenshtein_distance(&trace_1, &trace_2),
            6.0 / 4.0
        )
    }
}

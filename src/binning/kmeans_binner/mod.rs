mod _kmeans;

use super::Binner;
// It is called kmeans, but uses the KMeans++ initializer, so it is KMeans++
use _kmeans::kmeans;
use itertools::Itertools;

#[derive(Clone)]
pub struct KMeansArgs {
    k: usize,
    max_iter: usize,
    seed: Option<u64>,
}

impl Default for KMeansArgs {
    fn default() -> Self {
        Self {
            k: 3,
            max_iter: 100,
            seed: None,
        }
    }
}

impl KMeansArgs {
    pub fn new(k: usize, max_iter: usize, seed: Option<u64>) -> Self {
        Self { k, max_iter, seed }
    }
}

pub struct KMeansBinner {
    args: KMeansArgs,
    centroids: Vec<f64>,
}

impl Binner<f64> for KMeansBinner {
    type Args = KMeansArgs;

    fn new(data: Vec<f64>, args: KMeansArgs) -> Self {
        let data: Vec<Vec<f64>> = data.into_iter().map(|point| vec![point]).collect();
        let centroids: Vec<f64> = kmeans(args.k, &data, args.max_iter, args.seed)
            .centroids
            .into_iter()
            .map(|mut centroid| centroid.0.pop().unwrap())
            .sorted_by(|x, y| x.partial_cmp(y).unwrap())
            .collect();
        Self { centroids, args }
    }

    fn num_bins(&self) -> usize {
        self.args.k
    }

    fn bin(&self, data: f64) -> usize {
        self.centroids
            .iter()
            .map(|val| (data - val).abs())
            .enumerate()
            .min_by(|x, y| x.1.partial_cmp(&y.1).unwrap())
            .unwrap()
            .0
    }
}

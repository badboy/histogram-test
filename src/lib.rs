use serde::{Deserialize, Serialize};

/// A bucketing algorithm for histograms.
///
/// It's responsible to calculate the bucket a sample goes into.
/// It can calculate buckets on-the-fly or pre-calculate buckets and re-use that when needed.
pub trait Bucketing {
    /// Get the bucket's minimum value the sample falls into.
    fn sample_to_bucket_minimum(&self, sample: u64) -> u64;

    /// The computed bucket ranges for this bucketing algorithm.
    fn ranges(&self) -> &[u64];
}

/// A functional bucketing algorithm.
///
/// Bucketing is performed by a function, rather than pre-computed buckets.
/// The bucket index of a given sample is determined with the following function:
///
/// i = ⌊n log<sub>base</sub>(𝑥)⌋
///
/// In other words, there are n buckets for each power of `base` magnitude.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Functional {
    exponent: f64,
}

impl Functional {
    /// Instantiate a new functional bucketing.
    pub fn new(log_base: f64, buckets_per_magnitude: f64) -> Functional {
        let exponent = log_base.powf(1.0 / buckets_per_magnitude);

        Functional { exponent }
    }

    /// Maps a sample to a "bucket index" that it belongs in.
    /// A "bucket index" is the consecutive integer index of each bucket, useful as a
    /// mathematical concept, even though the internal representation is stored and
    /// sent using the minimum value in each bucket.
    fn sample_to_bucket_index(&self, sample: u64) -> u64 {
        ((sample + 1) as f64).log(self.exponent) as u64
    }

    /// Determines the minimum value of a bucket, given a bucket index.
    fn bucket_index_to_bucket_minimum(&self, index: u64) -> u64 {
        self.exponent.powf(index as f64) as u64
    }
}

impl Bucketing for Functional {
    fn sample_to_bucket_minimum(&self, sample: u64) -> u64 {
        if sample == 0 {
            return 0;
        }

        let index = self.sample_to_bucket_index(sample);
        self.bucket_index_to_bucket_minimum(index)
    }

    fn ranges(&self) -> &[u64] {
        unimplemented!("Bucket ranges for functional bucketing are not precomputed")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn regression_1623335() {
        let f = Functional::new(2.0, 8.0);

        let tests = vec![
            // (input, output)
            (7, 1),
            (8, 2),
            (9, 2),
            (10, 2),
            (11, 2),
            (12, 2),
            (13, 3),
            (14, 3),
            (15, 3),
            (16, 4),
        ];

        for (input, output) in tests {
            assert_eq!(output, f.bucket_index_to_bucket_minimum(input), "Input: {}, output: {}", input, output);
        }
    }
}

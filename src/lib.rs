#![warn(
    clippy::redundant_clone,
    clippy::doc_markdown,
    clippy::perf,
    missing_debug_implementations,
    rust_2018_idioms
)]

pub mod binning;
pub mod comparators;
pub mod distance;
pub mod emd;
pub mod utils;

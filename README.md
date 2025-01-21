A rust implementation of a permutation test approach for process hypothesis testing (PHT).

## Example

```rust
use pcomprs::comparators::{
    bootstrap::{
        bootstrap_comparator::BootstrapTestComparator, control_flow::ControlFlowBootstrapComparator,
    },
    permutation_test::{
        permutation_test_comparator::PermutationTestComparator,
        timed_levenshtein::TimedLevenshteinPermutationComparator,
    },
};
use process_mining::{import_xes_file, XESImportOptions};

let seed = 1337;
let log_1 = import_xes_file("path/to/log_1.xes.gz", XESImportOptions::default()).unwrap();
let log_2 = import_xes_file("path/to/log_2.xes.gz", XESImportOptions::default()).unwrap();

let permutation_result =
    TimedLevenshteinPermutationComparator::default().compare(&log_1, &log_2, 10_000);
println!(
    "Timed Control Flow Permutation Test: {}",
    permutation_result.pvalue
);

// Or:

let bootstrap_result = ControlFlowBootstrapComparator.compare(
    &log_1,
    &log_2,
    log_1.traces.len(),
    10_000,
    Some(seed),
);
println!("Control Flow Bootstrap Test: {}", bootstrap_result.pvalue);
```



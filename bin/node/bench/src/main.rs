#[macro_use] mod core;
mod import;

use crate::core::run_benchmark;
use import::ImportBenchmarkDescription;
use node_testing::bench::{Profile, KeyTypes};

fn main() {
    let filter = std::env::args().nth(1);

    sc_cli::init_logger("");

    let benchmarks = matrix!(
        profile in [Profile::Wasm, Profile::Native] =>
            ImportBenchmarkDescription {
                profile: *profile,
                key_types: KeyTypes::Sr25519,
            }
    );

    if filter.is_some() && filter.as_ref().unwrap() == "list" {
        for benchmark in benchmarks.iter() {
            println!("{}: {}", benchmark.name(), benchmark.path().full())
        }
    }

    for benchmark in benchmarks {
        if filter.as_ref().map(|f| benchmark.path().has(f)).unwrap_or(true) {
            run_benchmark(benchmark);
        }
    }
}
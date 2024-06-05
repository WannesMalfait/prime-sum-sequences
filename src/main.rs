use std::cmp::max;
use std::thread;
use std::time::Instant;

use primes::{self, find_prime_quadruplet, gen_primes_upto_n, Hankel};

use clap::Parser;

/// Brute force search for Hamiltonian cycles
///
/// Searches for cycles of length n, where n goes from
/// `start + offset` upto and including `maximum` in steps of `increment`.
/// Since cycles of odd length are impossible, `start` and `increment` should always be
/// even. If run with `m` threads, use `increment = 2 * m`.
///
/// `divisor` indicates where to start searching in the previous path. If the path is
/// length `n` then we start a backtracking search from index `n/divisor`. If `divisor`
/// is 0, then we start searching from index 1.
fn test_for_cycles(
    maximum: usize,
    start: usize,
    increment: usize,
    offset: usize,
    divisor: usize,
    primes: &[usize],
) {
    // When we try to create a new cycle
    let decrement = max(6, increment);
    // Create the first Hamiltonian cycle
    let mat = primes::Hankel::prime_sum_matrix(start + offset, Some(primes));
    let mut previous_path = mat
        .is_hamiltonian()
        .expect("No Hamiltonian cycle found for the starting index");
    let mut i = start + offset;
    while i <= maximum {
        let mat = Hankel::prime_sum_matrix(i, Some(primes));
        // We attempt to re-use the previous cycle by only changing the last
        // vertices in the cycle
        if !mat.hamiltonian_cycle(&mut previous_path, i - decrement) {
            // It didn't work -> create a new cycle from scratch
            let cycles_start = match divisor {
                0 => 1,
                _ => i / divisor,
            };
            if !mat.hamiltonian_cycle(&mut previous_path, cycles_start) {
                // Didn't find a cycle
                panic!("Did not find Hamiltonian cycle for size {}.", i);
            }
        }
        // Double check if it is actually a valid cycle
        if !mat.valid_cycle(&previous_path) {
            panic!("Generated invalid path");
        }
        // If the even index has a cycle then we can always remove one vertex
        // to create a valid path of length index - 1. Therefore we only check
        // the even indices.
        i += increment;
        previous_path.resize(previous_path.len() + increment, 0);
    }
}

fn test_for_cycles_naive(
    maximum: usize,
    start: usize,
    increment: usize,
    offset: usize,
    primes: &[usize],
) {
    let mut i = start + offset;
    while i <= maximum {
        if find_prime_quadruplet(i / 2, Some(primes)).is_none() {
            panic!("Did not find Hamiltonian cycle for size {}.", i);
        };
        // If the even index has a cycle then we can always remove one vertex
        // to create a valid path of length index - 1. Therefore we only check
        // the even indices.
        i += increment;
    }
}

/// Search for prime sum sequences.
#[derive(Parser, Debug)]
#[command(name= "Prime sum sequences", version, author, long_about=None)]
struct Cli {
    /// Sequence length to start at
    #[arg(short, long)]
    start: Option<usize>,
    /// Maximum sequence length to search for
    #[arg(short, long)]
    max: usize,
    /// Number of threads
    #[arg(short, long, default_value_t = 1)]
    threads: usize,
    /// Stack size in bytes
    #[arg(long, default_value_t = 1048576)]
    stack_size: usize,
    /// Greedily start at n/divisor if non-zero
    #[arg(short, long, default_value_t = 0)]
    divisor: usize,
    /// Use greedy fast search
    #[arg(short, long)]
    fast: bool,
}

fn main() {
    let cli = Cli::parse();
    let start = match cli.start {
        Some(arg) => {
            if arg < cli.threads * 2 {
                eprintln!("The number of threads must be less than the start/2");
                return;
            }
            if arg % 2 != 0 {
                eprintln!("The start should be even");
                return;
            }
            arg
        }
        None => max(cli.threads * 2, 12),
    };
    let increment = 2 * cli.threads;

    let now = Instant::now();

    // Calculate primes ahead of time.
    println!("Calculating primes");
    let primes = gen_primes_upto_n(2 * cli.max - 1);
    let primes = std::sync::Arc::new(primes);
    println!("Finished calculating primes in {:?}", now.elapsed());

    std::thread::scope(|s| {
        for i in 0..cli.threads {
            let builder = thread::Builder::new();
            builder
                // Spawn threads with explicit stack size
                // Needed because of the heavy recursion
                .stack_size(cli.stack_size)
                .spawn_scoped(s, {
                    let primes = primes.clone();
                    move || {
                        if cli.fast {
                            test_for_cycles_naive(cli.max, start, increment, i * 2, &primes);
                        } else {
                            test_for_cycles(cli.max, start, increment, i * 2, cli.divisor, &primes);
                        }
                    }
                })
                .unwrap();
        }
    });
    println!("All threads done, total time: {:?}", now.elapsed());
}

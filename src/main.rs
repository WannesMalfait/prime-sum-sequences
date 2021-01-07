use std::cmp::max;
use std::thread;
use std::time::Instant;

mod hankel;
use hankel::Hankel;

use clap::{App, Arg};

/// Brute force search for Hamiltonian cycles
///
/// Searches for cycles of length n, where n goes from
/// `start + offset` upto and including `maximum` in steps of `increment`.
/// Since cycles of odd length are impossible, `start` and `increment` should always be
/// even. If run with `m` threads, use `increment = 2 * m`.
///
/// `divisor` indicates where to start searching in the previous path. If the path is
/// length `n` then we start a backtracking search from index `n/divisor`. Should not be 0
fn test_for_cycles(maximum: usize, start: usize, increment: usize, offset: usize, divisor: usize) {
    // When we try to create a new cycle
    let decrement = max(6, increment);
    // Generate all the primes we need
    let primes = hankel::gen_primes_upto_n(2 * maximum - 1);
    // Create the first Hamiltonian cycle
    let mat = Hankel::prime_sum_matrix(start + offset, Some(&primes));
    let mut previous_path = mat
        .is_hamiltonian()
        .expect("No Hamiltonian cycle found for the starting index");
    let mut i = start + offset;
    let now = Instant::now();
    while i <= maximum {
        let mat = Hankel::prime_sum_matrix(i, Some(&primes));
        // We attempt to re-use the previous cycle by only changing the last
        // vertices in the cycle
        if !mat.hamiltonian_cycle(&mut previous_path, i - decrement) {
            // It didn't work -> create a new cycle from scratch
            println!(
                "Thread: {} creating cycle of length {} from scratch",
                offset / 2,
                i
            );
            let cycles_start = match divisor {
                0 => 1,
                _ => i / divisor,
            };
            if !mat.hamiltonian_cycle(&mut previous_path, cycles_start) {
                break; // Didn't find a cycle
            }
        }
        // Double check if it is actually a valid cycle
        if mat.valid_cycle(&previous_path) {
            println!("Thread: {} FOUND A VALID CYCLE OF LENGTH {}", offset / 2, i)
        } else {
            break;
        }
        // If the even index has a cycle then we can always remove one vertex
        // to create a valid path of length index - 1. Therefore we only check
        // the even indices.
        i += increment;
        for _ in 0..increment {
            previous_path.push(0);
        }
    }
    println!("Thread: {} took {:?}", offset / 2, now.elapsed());
}
fn main() {
    let matches = App::new("Prime sum sequences")
        .version("1.0")
        .author("Wannes M. <wannes.malfait@gmail.com>")
        .about("Bruteforce search for prime sum sequences")
        .arg(
            Arg::with_name("Start")
                .short("s")
                .long("start")
                .help("Start searching from this length")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("Maximum")
                .short("m")
                .long("max")
                .help("Search upto and including Maximum")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("Thread num")
                .short("t")
                .long("threads")
                .help("Number of threads")
                .default_value("1")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("Stack size")
                .long("stack")
                .help("Stack size in bytes")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("Divisor")
                .long("divisor")
                .short("d")
                .help("Greedily start at n/divisor if non-zero")
                .default_value("0")
                .takes_value(true),
        )
        .get_matches();
    let stack_size;
    if let Some(arg) = matches.value_of("Stack size") {
        stack_size = arg.parse().expect("Expected a number");
    } else {
        stack_size = 1024 * 1024;
    }
    let thread_num = matches
        .value_of("Thread num")
        .unwrap()
        .parse()
        .expect("Expected a number");
    let maximum = matches
        .value_of("Maximum")
        .unwrap()
        .parse()
        .expect("Expected a number");
    let start;
    if let Some(arg) = matches.value_of("Start") {
        start = arg.parse().expect("Expected a number");
        if start < thread_num * 2 {
            eprintln!("The number of threads must be less than the start/2");
            return;
        }
    } else {
        start = max(thread_num * 2, 12);
    }
    // Previously on fails the backtracking would start from scratch
    // Now we start from i/divisor
    // 2 Seems to give faster result, went from 54s -> 42s
    let divisor = matches
        .value_of("Divisor")
        .unwrap()
        .parse()
        .expect("Expected a number");
    let increment = 2 * thread_num;
    let mut threads = Vec::with_capacity(thread_num);
    // Spawn threads with explicit stack size
    // Needed because of the heavy recursion
    for i in 0..thread_num {
        let builder = thread::Builder::new();
        threads.push(
            builder
                .stack_size(stack_size)
                .spawn(move || test_for_cycles(maximum, start, increment, i * 2, divisor))
                .unwrap(),
        );
    }
    // Wait for threads to join
    for thread in threads {
        let _ = thread.join();
    }
}

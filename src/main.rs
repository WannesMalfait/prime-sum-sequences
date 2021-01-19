use std::cmp::max;
use std::thread;
use std::time::Instant;

use primes::{self, Hankel};

use clap::{App, Arg};

use pbr::{MultiBar, Pipe, ProgressBar};

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
    mut pb: ProgressBar<Pipe>,
) {
    // for calculating total time
    let now = Instant::now();
    // When we try to create a new cycle
    let decrement = max(6, increment);
    // Generate all the primes we need
    let primes = primes::gen_primes_upto_n(2 * maximum - 1);
    // Create the first Hamiltonian cycle
    let mat = primes::Hankel::prime_sum_matrix(start + offset, Some(&primes));
    let mut previous_path = mat
        .is_hamiltonian()
        .expect("No Hamiltonian cycle found for the starting index");
    let mut i = start + offset;
    while i <= maximum {
        let mat = Hankel::prime_sum_matrix(i, Some(&primes));
        // We attempt to re-use the previous cycle by only changing the last
        // vertices in the cycle
        if !mat.hamiltonian_cycle(&mut previous_path, i - decrement) {
            // It didn't work -> create a new cycle from scratch
            let cycles_start = match divisor {
                0 => 1,
                _ => i / divisor,
            };
            if !mat.hamiltonian_cycle(&mut previous_path, cycles_start) {
                break; // Didn't find a cycle
            }
        }
        // Double check if it is actually a valid cycle
        if !mat.valid_cycle(&previous_path) {
            break;
        }
        // If the even index has a cycle then we can always remove one vertex
        // to create a valid path of length index - 1. Therefore we only check
        // the even indices.
        i += increment;
        pb.inc();
        for _ in 0..increment {
            previous_path.push(0);
        }
    }
    pb.finish_print(&format!("Thread: {} took {:?}", offset / 2, now.elapsed()));
}
fn main() {
    let matches = App::new("Prime sum sequences")
        .version("1.0")
        .author("Wannes M. <wannes.malfait@gmail.com>")
        .about("Bruteforce search for prime sum sequences using backtracking.")
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
                .default_value("1048576") // 1024*1024 = 1 MiB or 1024 KiB
                .takes_value(true),
        )
        .arg(
            Arg::with_name("Divisor")
                .long("div")
                .short("d")
                .help("Greedily start at n/divisor if non-zero")
                .default_value("0")
                .takes_value(true),
        )
        .get_matches();
    let stack_size = matches
        .value_of("Stack size")
        .unwrap()
        .parse()
        .expect("Expected a number");
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
    // This works badly for small lengths (< 100)
    let divisor = matches
        .value_of("Divisor")
        .unwrap()
        .parse()
        .expect("Expected a number");
    let increment = 2 * thread_num;
    let mut threads = Vec::with_capacity(thread_num);

    let now = Instant::now();

    let mb = MultiBar::new();
    // Spawn threads with explicit stack size
    // Needed because of the heavy recursion
    for i in 0..thread_num {
        let builder = thread::Builder::new();
        let mut pb = mb.create_bar(((maximum - start + i * 2) / increment) as u64);
        pb.show_time_left = false;
        pb.show_percent = false;
        pb.show_tick = true;
        pb.tick_format("'`-._,_.-´'");
        threads.push(
            builder
                .stack_size(stack_size)
                .spawn(move || test_for_cycles(maximum, start, increment, i * 2, divisor, pb))
                .unwrap(),
        );
    }
    mb.listen();
    println!("All threads done, total time: {:?}", now.elapsed());
    // Wait for threads to join
    for thread in threads {
        let _ = thread.join();
    }
}

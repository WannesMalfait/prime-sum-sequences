use std::cmp::max;
use std::thread;
use std::time::{Duration, Instant};

mod hankel;
use hankel::Hankel;
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
            if !mat.hamiltonian_cycle(&mut previous_path, i / divisor) {
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
    const STACK_SIZE: usize = 1024 * 1024 * 4;
    const THREAD_NUM: usize = 6;
    let max = 4000;
    let start = 2000;
    let increment = 2 * THREAD_NUM;
    let divisor = 2; // Seems to give faster result, went from 54s -> 42s
    let mut threads = Vec::with_capacity(THREAD_NUM);
    // Spawn threads with explicit stack size
    // Needed because of the heavy recursion
    for i in 0..THREAD_NUM {
        let builder = thread::Builder::new();
        threads.push(
            builder
                .stack_size(STACK_SIZE)
                .spawn(move || test_for_cycles(max, start, increment, i * 2, divisor))
                .unwrap(),
        );
    }
    // Wait for threads to join
    for thread in threads {
        let _ = thread.join();
    }
}

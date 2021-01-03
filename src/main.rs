use std::io;
use std::thread;

mod hankel;
use hankel::Hankel;

fn test_for_cycles() -> io::Result<()> {
    const MAX: usize = 20000;
    // START should be bigger than 2*DECREMENT - 1
    const START: usize = 20000;
    // How many vertices from the previous cycle
    // do we change to try to create a new cycle?
    const DECREMENT: usize = 6;
    // Generate all the primes we need
    let primes = hankel::gen_primes_upto_n(2 * MAX - 1);
    // Create the first Hamiltonian cycle
    let mat = Hankel::prime_sum_matrix(START, Some(&primes));
    let mut previous_path = mat
        .is_hamiltonian()
        .expect("No Hamiltonian cycle found for the starting index");
    let mut i = START;
    'outer: while i <= MAX {
        let mat = Hankel::prime_sum_matrix(i, Some(&primes));
        let mut start_index = i - DECREMENT;
        // We attempt to re-use the previous cycle by only changing the last
        // vertices in the cycle
        while !mat.hamiltonian_cycle(&mut previous_path, start_index) {
            // If it didn't work create a new cycle
            if start_index <= i - DECREMENT * 2 + 1 {
                println!("Trying from 0");
                if !mat.hamiltonian_cycle(&mut previous_path, 1) {
                    break 'outer;
                } else {
                    break;
                }
            }
            println!(
                "Failed with index {}, now trying with {}",
                start_index,
                start_index - DECREMENT + 1
            );
            start_index -= DECREMENT - 1;
        }
        // Double check if it is actually a valid cycle
        if mat.valid_cycle(&previous_path) {
            println!("FOUND A VALID CYCLE OF LENGTH {}", i)
        } else {
            break;
        }
        // If the even index has a cycle then we can always remove one vertex
        // to create a valid path of length index - 1. Therefore we only check
        // the even indices.
        i += 2;
        previous_path.push(0);
        previous_path.push(0);
    }
    Ok(())
}
fn main() -> io::Result<()> {
    const STACK_SIZE: usize = 1024 * 1024 * 4;
    // Spawn thread with explicit stack size
    // Needed because of the heavy recursion
    let child = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(test_for_cycles)
        .unwrap();

    // Wait for thread to join
    child.join().unwrap()
}

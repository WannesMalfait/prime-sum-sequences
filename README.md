# About the repo

A CLI written in rust to find prime sum sequences.

## Prime sum sequence

A prime sum sequence of length n is a permutation of the numbers one to n such that the sum of two consecutive numbers is prime. The idea comes from 
[this video](https://www.youtube.com/watch?v=AXfl_e33Gt4)
by Matt Parker. Finding such a sequence is equivalent to finding a Hamiltonian path in the corresponding graph where vertices are adjacent if and only
if their sum is prime. The associated matrix of such a graph is a [Hankel matrix](https://encyclopediaofmath.org/wiki/Hankel_matrix), and the graph is therefore called
a Hankel graph.

### Examples

- 1-2-3-4-7-6-5-8-9-10
- 6-1-4-3-2-5

## Running

Clone the repository. The code is written in rust, you can build the binary using cargo and get usage help like so:
```term
cargo run --release -- --help
```
### Examples

To try to find if there are sequences of length 100 up to 1000, run:
```term
cargo run --release -- --max 1000 --start 100
```
If the code runs succesfully then a cycle was found for every n. Note that cycles only exist for even n, so `start` should always be even.

You can also run the search on multiple threads: 
```term
cargo run --release -- --max 2000 --start 100 --threads 4
```

## Faster approach

In the paper "[Hamiltonicity in Prime Sum Graphs](https://doi.org/10.1007/s00373-020-02241-1)" by Chen, HB., Fu, HL. and Guo, JY, it is shown that there are infinitely many sizes for which there is a Hamiltonian cycle. As part of this result they proved the following criterium:

If $p_1 < p_2 <= 2n$ are primes (allowing $p_1 = 1$) such that $p_1 + 2n$ and $p + 2n$ are prime, and $gcd((p_2 - p_1)/2, n) = 1$, then there is a prime sum cycle of length $2n$.

The `--fast` option checks for this criterium, instead of trying to brute-force a solution. For example, to check for sequences up to length `10000`, one can run
```term
cargo run --release -- --max 10000 --fast
```

On my laptop, checking the sequences up to length `100'000'000` on 8 threads took around 80 secs with this method. Of those 80 seconds, 60 were spent calculating prime numbers (not yet multithreaded).

### Constructing the sequence

This option doesn't actually compute the sequence. However, from the proof, a construction can be deduced which produces such a sequence. This is implemented in the `HamiltonianCycle` iterator. The reason it is implemented as an iterator, is because the whole sequence might otherwise not fit in memory. Printing the sequence is not (yet) exposed through the CLI.

## Feedback
Feel free to make pull requests or file issues.

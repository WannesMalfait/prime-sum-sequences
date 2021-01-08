# About the repo
This only tests for Hamiltonicity and doesn't generate all sequences. The search is done through backtracking. I have run the code to test for sequences till 10000.
I have not yet been able to prove mathematically that there is a sequence of length n, for every n. However the brute force calculations suggest strongly that this is
the case. I'm stil a beginner in rust, so the code might not be optimal.

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
```
cargo run --release -- -h
```
### Examples
To try to find if there are sequences of length 100 upto 1000, run:
```
cargo run --release -- --max 1000 --start 100
```
If the code runs succesfully then a cycle was found for every n. Note that cycles only exist for even n, so `start` should always be even.
You can also run the search on multiple threads: 
```
cargo run --release -- --max 2000 --start 100 --threads 4
```

## Feedback
Feel free to make pull requests or file issues.

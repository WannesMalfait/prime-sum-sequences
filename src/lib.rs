use std::borrow::Cow;
use std::io;
use std::io::Write;
use std::vec;

#[derive(Debug)]
/// A Hankel matrix is a matrix such that the entries along
/// a parallel to the main _anti-diagonal_ are equal. It
/// follows that the entries depend only on the sum i + j.
/// If i and j go from 0 to n - 1 we only need to store
/// 2n - 1 entries (i.e the first row and last column) to
/// have the full matrix.
///
/// # Example:
///
/// ```text
/// 0 1 0 1 0 1
/// 1 0 1 0 1 0
/// 0 1 0 1 0 0
/// 1 0 1 0 0 0
/// 0 1 0 0 0 1
/// 1 0 0 0 1 0
/// ```
///
/// This is the 6 by 6 Hankel matrix generated by calling
///
/// ```
/// // The primes upto 2*6 - 1 = 11
/// let primes = vec![2,3,5,7,11];
/// let mat = primes::Hankel::prime_sum_matrix(6, Some(&primes));
/// ```
pub struct Hankel {
    /// `diagonals` contains 2n-1 entries for an n by n matrix.
    diagonals: Vec<u8>,
    pub size: usize,
}

impl Hankel {
    /// Generate the Hankel matrix for the prime sum sequences of order n.
    ///
    /// `primes` should be generated at least upto 2n + 1, because we need to check if
    /// n + (n - 1) is prime
    pub fn prime_sum_matrix(n: usize, primes: Option<&[usize]>) -> Self {
        let mut diagonals = vec![0; 2 * n - 1];
        let mut i = 1; // index 0 is zero
        let p = match primes {
            Some(p) => Cow::Borrowed(p),
            None => Cow::Owned(gen_primes_upto_n(2 * n - 1)),
        };
        while i < 2 * n - 1 {
            if let Ok(_) = p.binary_search(&(i + 2)) {
                diagonals[i] = 1;
            }
            i += 2; // skip over the even numbers.
        }
        Self { diagonals, size: n }
    }
    /// Generate a Hankel matrix of size `n`by `n` from `values`
    /// Note that the rows and colums are 1-indexed, i.e the top
    /// left corner of the matrix is at index (1,1).
    pub fn from_sequence(n: usize, sequence: &[usize]) -> Self {
        let mut diagonals = vec![0; 2 * n - 1];
        for i in 0..(2 * n - 1) {
            if let Ok(_) = sequence.binary_search(&(i + 2)) {
                diagonals[i] = 1;
            }
        }
        Self { diagonals, size: n }
    }
    /// Get the entry in the matrix at the specified
    /// `row` and `col`. The first row and collumn
    /// are 1, i.e. the indexing starts at 1.
    pub fn get(&self, row: usize, col: usize) -> u8 {
        self.diagonals[row + col - 2]
    }
    /// Get the entry in the matrix at the specified
    /// `row` and `col`. The first row and collumn
    /// are 0, i.e. the indexing starts at 0.
    pub fn get_0_based(&self, row: usize, col: usize) -> u8 {
        self.diagonals[row + col]
    }
    /// Checks if `path` is a valid Hamiltonian path in
    /// the current graph.
    pub fn valid_path(&self, path: &[usize]) -> bool {
        for i in 0..(path.len() - 1) {
            match self.get(path[i], path[i + 1]) {
                0 => return false,
                _ => continue,
            }
        }
        true
    }
    /// Checks if `cycle` is a valid Hamiltonian cycle in
    /// the current graph.
    pub fn valid_cycle(&self, cycle: &[usize]) -> bool {
        for i in 0..(cycle.len() - 1) {
            match self.get(cycle[i], cycle[i + 1]) {
                0 => return false,
                _ => continue,
            }
        }
        self.get(cycle[0], cycle[cycle.len() - 1]) != 0
    }
    /// If there is a cycle return it. Otherwise return None.
    pub fn is_hamiltonian(&self) -> Option<Vec<usize>> {
        let mut path = vec![0; self.size];
        path[0] = 1;
        if self.hamiltonian_cycle(&mut path, 1) {
            Some(path)
        } else {
            None
        }
    }
    /// Tries to make a Hamiltonian cycle out of `path` using backtracking
    ///
    /// The values in the path before `pos` are left unchanged.
    /// Returns false if no cycle was constructed.
    pub fn hamiltonian_cycle(&self, path: &mut [usize], pos: usize) -> bool {
        if pos == self.size {
            // println!("cur length {}", cur_length);
            return self.get(path[0], path[pos - 1]) != 0;
        }
        // the sequence alternates between odd and even
        // loop backwards, because we are reusing the previously found cycles
        // which are all made up of smaller numbers
        let mut n = self.size - (pos + 1) % 2;
        'outer: while n > 1 {
            if self.get(path[pos - 1], n) == 0 {
                n -= 2;
                continue;
            }
            let mut j = pos % 2;
            while j < pos {
                if path[j] == n {
                    n -= 2;
                    continue 'outer;
                }
                j += 2;
            }
            path[pos] = n;
            if self.hamiltonian_cycle(path, pos + 1) {
                return true;
            }
            path[pos] = 0;
            n -= 2;
        }
        false
    }
    /// Prints the associated adjacency matrix to stdout.
    pub fn print(&self) -> io::Result<()> {
        let mut output = io::BufWriter::new(io::stdout());
        for row in 0..self.size {
            for col in 0..(self.size - 1) {
                write!(&mut output, "{}, ", self.get_0_based(row, col))?;
            }
            writeln!(&mut output, "{}", self.get_0_based(row, self.size - 1))?;
        }
        output.flush()?;
        Ok(())
    }
    /// Returns the degrees of all of the vertices in the graph.
    ///
    /// If the current `size` is n, then the returned vector has
    /// length n.
    ///
    /// The degrees are recalculated at every call.
    pub fn vertex_degrees(&self) -> Vec<usize> {
        let mut degrees = Vec::with_capacity(self.size);
        if self.size == 0 {
            return degrees;
        }
        let mut first = self.diagonals[0] as usize;
        let mut sum = first as usize;
        // Calculate the degree of the first vertex
        for i in 1..self.size {
            sum += self.diagonals[i] as usize;
        }
        degrees.push(sum);
        // The degree of the i-th vertex is the sum over the entries
        // in the diagonals array from i to i + n, where n is self.size
        // so if the diagonals array is something like:
        // [0,1,0,1,0,1,0,0,0,1,0]
        // then the degree of the vertices are:
        // 1st: 0+1+0+1+0+1 = 3
        // 2nd: 1+0+1+0+1+0 = 3
        // 3rd: 0+1+0+1+0+0 = 2
        // 4th: 1+0+1+0+0+0 = 2
        // 5th: 0+1+0+0+0+1 = 2
        // 6th: 1+0+0+0+1+0 = 2
        for i in self.size..self.diagonals.len() {
            sum += self.diagonals[i] as usize;
            sum -= first;
            first = self.diagonals[i - self.size + 1] as usize;
            degrees.push(sum);
        }
        degrees
    }
}

/// An iterator over a Hamiltonian path in the prime sum
/// graph of the given order.
pub struct HamiltonianPath {
    difference1: usize,
    difference2: usize,
    /// Half the size of the graph.
    half_size: usize,
    /// The current vertex in the path.
    current: usize,
}

impl HamiltonianPath {
    /// The number of vertices in the graph is: 2n = 2 * half_size.
    /// The two primes should be such that p1 + 2n and p2 + 2n
    /// are both prime, and such that gcd((p2-p1)/2, n) = 1.
    pub fn new(prime1: usize, prime2: usize, half_size: usize) -> Self {
        Self {
            difference1: (prime1 - 1) / 2,
            difference2: (prime2 - 1) / 2,
            half_size,
            current: 0,
        }
    }

    fn x_j(&self, j: usize) -> usize {
        2 * j - 1
    }

    fn y_k(&self, k: usize) -> usize {
        2 * self.half_size - 2 * (k - 1)
    }

    // x_j = 2 j - 1, so j = (x_j + 1) / 2;
    fn j_from_x_j(&self, x_j: usize) -> usize {
        (x_j + 1) / 2
    }

    fn k_from_y_k(&self, y_k: usize) -> usize {
        self.half_size - y_k / 2 + 1
    }
}

impl Iterator for HamiltonianPath {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == 0 {
            // We always start with 1.
            self.current = 1;
            return Some(1);
        }
        let is_odd = self.current % 2 == 1;
        if is_odd {
            let j = self.j_from_x_j(self.current);
            // Add self.half_size to prevent subtraction overflow.
            let mut k = (j + self.half_size - self.difference1) % self.half_size;
            if k == 0 {
                k = self.half_size;
            }
            let y_k = self.y_k(k);
            if y_k == 1 {
                // Back to the first vertex.
                return None;
            }
            self.current = y_k;
            Some(y_k)
        } else {
            let k = self.k_from_y_k(self.current);
            let mut j = (k + self.difference2) % self.half_size;
            if j == 0 {
                j = self.half_size;
            }
            let x_j = self.x_j(j);
            if x_j == 1 {
                // Back to the first vertex.
                return None;
            }
            self.current = x_j;
            Some(x_j)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.half_size * 2))
    }
}

/// Finds primes p1 < p2 <= 2 * n (with n == half_size) such that:
/// - p1 + 2 * n is prime
/// - p2 + 2 * n is prime
/// - gcd((p1 + p2)/2, n)  = 1
/// If no such primes exist, `None` is returned.
///
/// `primes` should contain all the primes from 2 up to 4 * n.
/// `half_size` should be at least 2.
///
/// NOTE: we allow p1 to be equal to 1.
pub fn find_prime_quadruplet(half_size: usize, primes: Option<&[usize]>) -> Option<(usize, usize)> {
    assert!(half_size >= 2);
    let all_primes = match primes {
        Some(p) => Cow::Borrowed(p),
        None => Cow::Owned(gen_primes_upto_n(4 * half_size)),
    };
    let half_index = match all_primes.binary_search(&(half_size * 2)) {
        Ok(n) => n,
        Err(n) => n,
    };

    let (iter_primes, bigger_primes) = all_primes.split_at(half_index);
    let mut first_prime_index = 0;
    for &p1 in std::iter::once(&1).chain(iter_primes.iter()) {
        // Check if p1 + 2 * n is prime.
        if bigger_primes.binary_search(&(p1 + 2 * half_size)).is_err() {
            first_prime_index += 1;
            continue;
        }
        for &p2 in iter_primes.iter().skip(first_prime_index) {
            if gcd((p2 - p1) / 2, half_size) != 1 {
                continue;
            }
            if bigger_primes.binary_search(&(p2 + half_size * 2)).is_err() {
                continue;
            }
            return Some((p1, p2));
        }
        first_prime_index += 1;
    }
    None
}

/// Compute the greatest common divisor of `a` and `b`.
fn gcd(mut a: usize, mut b: usize) -> usize {
    if a == b {
        return a;
    }
    if b > a {
        std::mem::swap(&mut a, &mut b);
    }
    while b > 0 {
        let temp = a;
        a = b;
        b = temp % b;
    }
    a
}

/// Generates the primes upto and including `n`. Doesn't
/// check for overflow on `n`
pub fn gen_primes_upto_n(n: usize) -> Vec<usize> {
    let mut primes = Vec::new();
    'outer: for i in 2..(n + 1) {
        for &prime in &primes {
            if i % prime == 0 {
                continue 'outer;
            }
        }
        primes.push(i);
    }
    primes
}

#[cfg(test)]
#[test]
fn correct_access() {
    let mat = Hankel::from_sequence(5, &[4, 6, 8]);
    let _ = mat.print();
    assert_eq!(mat.vertex_degrees(), vec![2, 2, 3, 2, 2]);
    assert!(!mat.valid_path(&[1, 3, 5, 2, 4]));
    assert_eq!(mat.get_0_based(0, 0), 0);
    assert_eq!(mat.get_0_based(1, 1), 1);
    assert_eq!(mat.get_0_based(2, 4), 1);
}

#[test]
fn hamilton() {
    let mat = Hankel::from_sequence(7, &[4, 7, 8]);
    assert!(mat.valid_path(&[7, 1, 6, 2, 5, 3, 4]));
    assert!(!mat.valid_cycle(&[7, 1, 6, 2, 5, 3, 4]));
}

#[test]
fn indexing_correct() {
    let test = HamiltonianPath::new(3, 17, 10);
    for i in 1..=10 {
        assert_eq!(i, test.j_from_x_j(test.x_j(i)));
        assert_eq!(i, test.k_from_y_k(test.y_k(i)));
    }
}

#[test]
fn path_length() {
    let test = HamiltonianPath::new(3, 17, 10);
    assert_eq!(test.count(), 20);
}

#[test]
fn prime_quadruplet() {
    assert_eq!(find_prime_quadruplet(10, None), Some((3, 17)));
}

#[test]
fn first_100() {
    let primes = gen_primes_upto_n(200);
    for half_size in 2..50 {
        let (p1, p2) = match find_prime_quadruplet(half_size, Some(&primes)) {
            Some(t) => t,
            None => panic!(),
        };
        let path = HamiltonianPath::new(p1, p2, half_size);
        assert_eq!(path.count(), half_size * 2);
    }
}

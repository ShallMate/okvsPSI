//! Utility functions and structs.

use rand::Rng;
use crate::Block;

const PROMPT_LENGTH: usize = 30;

/// Clone with a seed.
///
/// Usually used when a protocol executor needs to be cloned but we need each clone to execute its protocol with different random generator.
pub trait SeededClone {
    /// Clone with a seed.
    fn seeded_clone(&self, seed: usize) -> Self;
}

impl<T> SeededClone for T
where
    T: Clone,
{
    fn seeded_clone(&self, _seed: usize) -> Self {
        self.clone()
    }
}

/// Reverse a usize of specified length.
/// For example, reverse(11, 4) = 13, but reverse(11, 5) = 26.
pub fn reverse_usize(x: usize, bits: usize) -> usize {
    if bits == 0 {
        return 0;
    }
    let rb = x.reverse_bits();
    rb >> (usize::BITS as usize - bits)
}

/// Returns log2(s), where s is the smallest power of 2 greater than or equal to x.
pub fn log2ceil(x: usize) -> usize {
    if x.count_ones() == 1 {
        x.trailing_zeros() as usize
    } else {
        (usize::BITS - x.leading_zeros()) as usize
    }
}

/// Returns xor of two byte arrays.
pub fn xor_u8s(a: &[u8], b: &[u8]) -> Vec<u8> {
    assert_eq!(a.len(), b.len());
    let mut result = Vec::with_capacity(a.len());
    for i in 0..a.len() {
        result.push(a[i] ^ b[i]);
    }
    result
}

/// Ceil div
pub fn ceil_div(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

/// Round `a` up to a multiple of `b`.
pub fn round_up_to_multiple(a: usize, b: usize) -> usize {
    ceil_div(a, b) * b
}

/// Random permute a vector of usizes
pub fn random_permute_usize_vec(vec: &mut Vec<usize>) {
    let mut rng = rand::thread_rng();
    for i in 0..vec.len() {
        let j = rng.gen_range(i..vec.len());
        vec.swap(i, j);
    }
}

/// Generate non-repeating list, items within range n.
pub fn generate_non_repeating_list(n: usize, count: usize) -> Vec<usize> {
    assert!(n >= count);
    if n == count {
        let mut vec = (0..n).collect::<Vec<_>>();
        random_permute_usize_vec(&mut vec);
        vec
    } else {
        let mut rng = rand::thread_rng();
        let mut result = (0..count)
            .map(|_| rng.gen_range(0..n - count))
            .collect::<Vec<_>>();
        result.sort();
        for i in 0..count {
            result[i] += i;
        }
        random_permute_usize_vec(&mut result);
        result
    }
}

/// XOR x with y in place. Lengths must match.
#[inline]
pub fn xor_u8s_inplace(x: &mut [u8], y: &[u8]) {
    assert_eq!(x.len(), y.len());
    for i in 0..x.len() {
        x[i] ^= y[i];
    }
}

/// XOR a block array of fixed size inplace
#[inline]
pub fn blockc_xor_inplace<const C: usize>(a: &mut [Block; C], b: &[Block; C]) {
    for i in 0..C {a[i] ^= b[i];}
}

/// XOR a block array in place
#[inline]
pub fn blocks_xor_inplace(a: &mut [Block], b: &[Block]) {
    assert_eq!(a.len(), b.len());
    for i in 0..a.len() {a[i] ^= b[i];}
}

/// XOR a block array with fixed size
#[inline]
pub fn blockc_xor<const C: usize>(a: &[Block; C], b: &[Block; C], c: &mut [Block; C]) {
    for i in 0..C {c[i] = a[i] ^ b[i];}
}

/// XOR a block array
#[inline]
pub fn blocks_xor(a: &[Block], b: &[Block], c: &mut [Block]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), c.len());
    for i in 0..a.len() {c[i] = a[i] ^ b[i];}
}

/// Format and print time. The `prompt` is a string put before the colon. The `tabs * 2` are how many spaces to put before prompt.
/// If `div > 1`, will print an "average time" and a "total time".
pub fn print_time(prompt: &str, tabs: usize, total_time: std::time::Duration, div: usize) {
    let time = total_time / (div as u32);
    // print spaces = tabs * 2
    for _ in 0..tabs {
        print!("  ");
    }
    // print prompt
    print!("{}", prompt);
    // fill spaces
    if PROMPT_LENGTH > prompt.len() + tabs * 2 {
        for _ in 0..(PROMPT_LENGTH - prompt.len() - tabs * 2) {
            print!(" ");
        }
    }
    if time <= std::time::Duration::new(0, 1000) {
        print!(": {:>9} ns", time.as_nanos());
    } else if time <= std::time::Duration::new(0, 1000000) {
        print!(": {:>9.3} us", time.as_nanos() as f64 / 1000.0);
    } else if time <= std::time::Duration::new(0, 1000000000) {
        print!(": {:>9.3} ms", time.as_micros() as f64 / 1000.0);
    } else {
        print!(": {:>9.3} s ", time.as_millis() as f64 / 1000.0);
    }
    if div > 1 {
        let time = total_time;
        if time <= std::time::Duration::new(0, 1000) {
            print!(" (total {:>9} ns", time.as_nanos());
        } else if time <= std::time::Duration::new(0, 1000000) {
            print!(" (total {:>9.3} us", time.as_nanos() as f64 / 1000.0);
        } else if time <= std::time::Duration::new(0, 1000000000) {
            print!(" (total {:>9.3} ms", time.as_micros() as f64 / 1000.0);
        } else {
            print!(" (total {:>9.3} s ", time.as_millis() as f64 / 1000.0);
        }
        print!(", {} times)", div);
    } 
    println!();
}

/// A utility struct that allows tracking multiple timers.
/// 
/// The user needs to register a timer with [`Timer::register`] to get a handle.
/// After that, the user could use [`Timer::tick`] and [`Timer::tock`] to measure a time interval.
/// The interval is accumulated between the pair of calls. Finally, the user could use [`Timer::print`]
/// or [`Timer::print_div`] to print the accumulated time or averaged time.
/// The user could use [`Timer::clear`] to clear all timers.
pub struct Timer {
    start: Vec<std::time::Instant>,
    accumulated: Vec<std::time::Duration>,
    name: Vec<String>,
    tabs: usize,
}
impl Timer {
    /// Create a new timer.
    pub fn new() -> Self {
        Self {
            start: vec![],
            accumulated: vec![],
            name: vec![],
            tabs: 0,
        }
    }
    /// Set the tabs of the timer. See [`print_time`] method for more information.
    pub fn tabs(self, tabs: usize) -> Self {
        Self {
            start: self.start,
            accumulated: self.accumulated,
            name: self.name,
            tabs,
        }
    }
    /// Register a timer with a name. Returns a handle to be used with [`Timer::tick`] and [`Timer::tock`].
    /// Note that when you register, a [`Timer::tick`] is automatically called. Therefore, if you need only
    /// record one interval, you could directly call [`Timer::tock`] after [`Timer::register`].
    pub fn register(&mut self, name: &str) -> usize {
        self.start.push(std::time::Instant::now());
        self.accumulated.push(std::time::Duration::new(0, 0));
        self.name.push(name.to_string());
        self.start.len() - 1
    }
    /// Starts the timer with the given handle.
    pub fn tick(&mut self, index: usize) {
        self.start[index] = std::time::Instant::now();
    }
    /// Stops the timer with the given handle. The time interval from the previous call of [`Timer::tick`] is accumulated.
    pub fn tock(&mut self, index: usize) {
        self.accumulated[index] += self.start[index].elapsed();
    }
    /// Print the accumulated time of all timers.
    pub fn print(&self) {
        for i in 0..self.start.len() {
            let acc = &self.accumulated[i];
            print_time(&self.name[i], self.tabs, *acc, 1);
        }
    }
    /// Print the accumulated time of all timers, divided by `div` (averaged time).
    pub fn print_div(&self, div: usize) {
        for i in 0..self.start.len() {
            let acc = self.accumulated[i];
            print_time(&self.name[i], self.tabs, acc, div);
        }
    }
    /// Clear all timers. Semantically equivalent to creating a new timer.
    pub fn clear(&mut self) {
        self.start.clear();
        self.accumulated.clear();
        self.name.clear();
    }
}

/// A utility struct that allows tracking a single timer.
/// 
/// This is similar to [`Timer`] because you could call [`TimerSingle::tick`] and [`TimerSingle::tock`] multiple times.
/// But it tracks only one timer. The name of the timer is only needed when printing.
pub struct TimerSingle {
    start: std::time::Instant,
    accumulated: std::time::Duration,
    tabs: usize,
}
impl TimerSingle {
    /// Create a new timer.
    /// Note that when you create, a [`TimerSingle::tick`] is automatically called. Therefore, if you need only
    /// record one interval, you could directly call [`TimerSingle::tock`] after creation.
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
            accumulated: std::time::Duration::new(0, 0),
            tabs: 0,
        }
    }
    /// Set the tabs of the timer. See [`print_time`] method for more information.
    pub fn tabs(self, tabs: usize) -> Self {
        Self {
            start: self.start,
            accumulated: self.accumulated,
            tabs: tabs,
        }
    }
    /// Starts the timer.
    pub fn tick(&mut self) {
        self.start = std::time::Instant::now();
    }
    /// Stops the timer. The time interval from the previous call of [`TimerSingle::tick`] is accumulated.
    pub fn tock(&mut self) {
        self.accumulated += self.start.elapsed();
    }
    /// Print the accumulated time of the timer.
    pub fn print(&self, name: &str) {
        let acc = &self.accumulated;
        print_time(name, self.tabs, *acc, 1);
    }
    /// This is simply a combination of [`TimerSingle::tock`] and [`TimerSingle::print`].
    /// Useful if you need only record one interval.
    pub fn finish(mut self, name: &str) {
        self.tock();
        self.print(name);
    }
    /// Print the accumulated time of the timer, divided by `div` (averaged time).
    pub fn print_div(&self, name: &str, div: usize) {
        let acc = self.accumulated;
        print_time(&name, self.tabs, acc, div);
    }
}

/// A utility struct that allows measuing a time interval.
/// 
/// User simply creates a new [`TimerOnce`] and calls [`TimerOnce::finish`] (or [`TimerOnce::finish_div`]) to print the time interval or averaged time interval.	
pub struct TimerOnce {
    start: std::time::Instant,
    tabs: usize,
}
impl TimerOnce {
    /// Create a new timer.
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
            tabs: 0,
        }
    }
    /// Set the tabs of the timer. See [`print_time`] method for more information.
    pub fn tabs(self, tabs: usize) -> Self {
        Self {
            start: self.start,
            tabs: tabs,
        }
    }
    /// Print the time interval.
    pub fn finish(self, prompt: &str) {
        let elapsed = self.start.elapsed();
        print_time(prompt, self.tabs, elapsed, 1);
    }
    /// Print the averaged time interval.
    pub fn finish_div(self, prompt: &str, div: usize) {
        let elapsed = self.start.elapsed();
        print_time(prompt, self.tabs, elapsed, div);
    }
}

/// Trait to indicate the object can be XORed inplace.
/// 
/// This trait is used to provide automatic implementation from [`crate::RandomOtSender`], [`crate::RandomOtReceiver`] to
/// [`crate::ChosenOtSender`], [`crate::ChosenOtReceiver`]. See the crate level documentation for this automatic implementation.
pub trait OtXorInplace {
    /// XOR inplace.
    fn xor_inplace(&mut self, other: &Self);
}

impl OtXorInplace for Block {
    fn xor_inplace(&mut self, other: &Self) {
        *self ^= *other;
    }
}

impl OtXorInplace for Vec<u8> {
    fn xor_inplace(&mut self, other: &Self) {
        assert_eq!(self.len(), other.len());
        for (a, b) in self.iter_mut().zip(other.iter()) {
            *a ^= *b;
        }
    }
}

impl OtXorInplace for u64 {
    fn xor_inplace(&mut self, other: &Self) {
        *self ^= *other;
    }
}

impl OtXorInplace for u8 {
    fn xor_inplace(&mut self, other: &Self) {
        *self ^= *other;
    }
}

/// Format and print communication. The `name` is a string put before the colon. The `tabs * 2` are how many spaces to put before prompt.
/// If `div > 1`, will print an "average comm" and a "total comm".
pub fn print_communication(name: &str, tabs: usize, bytes: usize, div: usize) {
    // print spaces = tabs * 2
    for _ in 0..tabs {
        print!("  ");
    }
    // print prompt
    print!("{}", name);
    // fill spaces
    if PROMPT_LENGTH > name.len() + tabs * 2 {
        for _ in 0..(PROMPT_LENGTH - name.len() - tabs * 2) {
            print!(" ");
        }
    }
    if bytes / div <= 4 {
        let bits = bytes as f64 * 8.0 / div as f64;
        print!(": {:>9.3} bt", bits);
    } else if bytes / div < 1024 {
        print!(": {:>9.3} B ", bytes / div);
    } else if bytes / div < 1024 * 1024 {
        print!(": {:>9.3} KB", bytes as f64 / 1024.0 / div as f64);
    } else {
        print!(": {:>9.3} MB", bytes as f64 / 1024.0 / 1024.0 / div as f64);
    }
    if div > 1 {
        if bytes <= 4 {
            let bits = bytes * 8;
            print!(" (total {:>9} bt", bits);
        } else if bytes < 1024 {
            print!(" (total {:>9} B ", bytes);
        } else if bytes < 1024 * 1024 {
            print!(" (total {:>9.3} KB", bytes as f64 / 1024.0);
        } else {
            print!(" (total {:>9.3} MB", bytes as f64 / 1024.0 / 1024.0);
        }
        print!(", {} times)", div);
    }
    println!();
}

#[cfg(test)]
pub mod tests {
    use crate::Block;

    use super::*;

    pub fn rand_u8s(len: usize) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        (0..len).map(|_| rng.gen()).collect()
    }

    pub fn rand_u8ss(dim0: usize, dim1: usize) -> Vec<Vec<u8>> {
        (0..dim0).map(|_| rand_u8s(dim1)).collect()
    }

    pub fn rand_u8_pair(len: usize) -> (Vec<u8>, Vec<u8>) {
        (rand_u8s(len), rand_u8s(len))
    }

    pub fn rand_u8_pairs(count: usize, u8_len: usize) -> Vec<(Vec<u8>, Vec<u8>)> {
        (0..count).map(|_| rand_u8_pair(u8_len)).collect()
    }

    pub fn rand_u8_pairs_ragged(count: usize, u8_max_len: usize) -> Vec<(Vec<u8>, Vec<u8>)> {
        (0..count)
            .map(|_| rand_u8_pair(rand::random::<usize>() % u8_max_len))
            .collect()
    }

    pub fn rand_usizes_mod(len: usize, modulus: usize) -> Vec<usize> {
        let mut rng = rand::thread_rng();
        (0..len).map(|_| rng.gen_range(0..modulus)).collect()
    }

    pub fn rand_u64() -> u64 {
        rand::thread_rng().gen()
    }

    pub fn rand_u64s(len: usize) -> Vec<u64> {
        let mut rng = rand::thread_rng();
        (0..len).map(|_| rng.gen()).collect()
    }

    pub fn rand_u64_pair() -> (u64, u64) {
        (rand_u64(), rand_u64())
    }

    pub fn rand_u64_pairs(count: usize) -> Vec<(u64, u64)> {
        (0..count).map(|_| rand_u64_pair()).collect()
    }

    pub fn rand_u128() -> u128 {
        rand::thread_rng().gen()
    }

    pub fn rand_u128s(len: usize) -> Vec<u128> {
        let mut rng = rand::thread_rng();
        (0..len).map(|_| rng.gen()).collect()
    }

    pub fn rand_u128_pair() -> (u128, u128) {
        (rand_u128(), rand_u128())
    }

    pub fn rand_u128_pairs(count: usize) -> Vec<(u128, u128)> {
        (0..count).map(|_| rand_u128_pair()).collect()
    }

    pub fn rand_block() -> Block {
        rand_u128().into()
    }

    pub fn rand_blocks(len: usize) -> Vec<Block> {
        rand_u128s(len).into_iter().map(|x| x.into()).collect()
    }

    pub fn rand_block_pair() -> (Block, Block) {
        (rand_block(), rand_block())
    }

    pub fn rand_block_pairs(count: usize) -> Vec<(Block, Block)> {
        (0..count).map(|_| rand_block_pair()).collect()
    }

    #[test]
    fn test_ceil_two_power() {
        assert_eq!(log2ceil(0), 0);
        assert_eq!(log2ceil(1), 0);
        assert_eq!(log2ceil(2), 1);
        assert_eq!(log2ceil(3), 2);
        assert_eq!(log2ceil(4), 2);
        assert_eq!(log2ceil(5), 3);
        assert_eq!(log2ceil(6), 3);
        assert_eq!(log2ceil(7), 3);
        assert_eq!(log2ceil(8), 3);
        assert_eq!(log2ceil(9), 4);
    }
}

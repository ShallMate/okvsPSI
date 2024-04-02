//! Provides BitString that behaves like [`Vec<bool>`] but saves memory.
//!
//! The bits are stored in a [`Vec<usize>`].

use rand::Rng;
use crate::Block;
pub(crate) type Storage = usize;
pub(crate) const ITEM_BITS: usize = std::mem::size_of::<Storage>() * 8;
pub(crate) const ITEM_BITS_LOG2: u32 = ITEM_BITS.trailing_zeros();
pub(crate) const ITEM_BITS_MASK: usize = ITEM_BITS - 1;

use crate::{hash::Hashable, utils::log2ceil};

/// A bit string that behaves like [`Vec<bool>`] but saves memory.
/// The bits are stored in a [`Vec<usize>`].
#[derive(Clone)]
pub struct BitString {
    data: Vec<Storage>,
    len: usize,
}

fn ceil_div(x: usize, y: usize) -> usize {
    if x % y == 0 {
        x / y
    } else {
        x / y + 1
    }
}

impl BitString {

    /// From raw parts
    pub fn from_raw_parts(data: Vec<Storage>, len: usize) -> Self {
        Self {
            data,
            len,
        }
    }

    /// An empty BitString.
    pub fn new() -> Self {
        Self::new_zeros(0)
    }

    /// A BitString with specified length, all set to 0.
    pub fn new_zeros(bit_length: usize) -> Self {
        Self {
            data: vec![0; ceil_div(bit_length, ITEM_BITS)],
            len: bit_length,
        }
    }

    /// A BitString with specified length, all set to 1.
    pub fn new_ones(bit_length: usize) -> Self {
        let mut ret = Self::new_zeros(bit_length);
        ret.not_inplace();
        ret
    }

    /// A Bitstring with `zeros` consecutive zeros and then `ones` consecutive ones.
    pub fn new_zeros_ones(zeros: usize, ones: usize) -> Self {
        let total_chunks = ceil_div(zeros + ones, ITEM_BITS);
        let split_chunk_index = zeros >> ITEM_BITS_LOG2;
        let mut data = vec![0; total_chunks];
        for i in split_chunk_index+1 .. total_chunks {
            data[i] = !0;
        }
        if split_chunk_index < total_chunks {
            data[split_chunk_index] = (!0) << (zeros & ITEM_BITS_MASK);
        }
        let mut ret = Self {
            data,
            len: zeros + ones,
        };
        ret.ensure_last_chunk();
        ret
    }

    /// A Bitstring with `ones` consecutive ones and then `zeros` consecutive zeros.
    pub fn new_ones_zeros(ones: usize, zeros: usize) -> Self {
        let total_chunks = ceil_div(ones + zeros, ITEM_BITS);
        let split_chunk_index = ones >> ITEM_BITS_LOG2;
        let mut data = vec![0; total_chunks];
        for i in 0..split_chunk_index {
            data[i] = !0;
        }
        if split_chunk_index < total_chunks {
            data[split_chunk_index] = (1 << (ones & ITEM_BITS_MASK)) - 1;
        }
        let ret = Self {
            data,
            len: zeros + ones,
        };
        ret

    }

    /// Ensure in the last chunk, the higher bits out of len scope are set to zero
    fn ensure_last_chunk(&mut self) {
        let last_chunk_bits = self.len & ITEM_BITS_MASK;
        if last_chunk_bits > 0 {
            let last_chunk = self.data.last_mut().unwrap();
            *last_chunk &= (1 << last_chunk_bits) - 1;
        }
    }

    /// A BitString with specified capacity. The length is 0, but it is prepared to be pushed `bit_capacity` bits.
    pub fn with_capacity(bit_capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(ceil_div(bit_capacity, ITEM_BITS)),
            len: 0,
        }
    }

    /// A random BitString with specified length.
    pub fn new_random(bit_length: usize) -> Self {
        let mut rng = rand::thread_rng();
        let data = (0..ceil_div(bit_length, ITEM_BITS))
            .map(|i| {
                let x = rng.gen();
                let remaining_bits = bit_length - i * ITEM_BITS;
                if remaining_bits < ITEM_BITS {
                    x & ((1 << remaining_bits) - 1)
                } else {
                    x
                }
            })
            .collect::<Vec<_>>();
        Self {
            data,
            len: bit_length,
        }
    }

    /// Create a BitString from a usize. Little endian.
    /// For example, usize `13` -> BitString `1011`
    pub fn from_usize(x: usize, bit_length: usize) -> Self {
        let mut ret = Self::new_zeros(bit_length);
        for i in 0..bit_length {
            ret.set(i, (x >> i) & 1 == 1);
        }
        ret
    }

    /// Create a bitstring from a string of `0` and `1`'s.
    /// This is not efficient, and is only for test convenience.
    pub fn from_string(s: &str) -> Self {
        let mut ret = Self::with_capacity(s.len());
        for c in s.chars() {
            if c == '0' {
                ret.push(false);
            } else {
                ret.push(true);
            }
        }
        ret
    }

    /// Set one bit.
    #[inline]
    pub fn set(&mut self, index: usize, bit: bool) {
        debug_assert!(
            index < self.len,
            "Index {} must be smaller than len {}.",
            index,
            self.len
        );
        if bit {
            self.data[index >> ITEM_BITS_LOG2] |= (1 as Storage) << (index & ITEM_BITS_MASK);
        } else {
            self.data[index >> ITEM_BITS_LOG2] &= !((1 as Storage) << (index & ITEM_BITS_MASK));
        }
    }

    /// Get one bit.
    #[inline]
    pub fn get(&self, index: usize) -> bool {
        debug_assert!(
            index < self.len,
            "Index {} must be smaller than len {}.",
            index,
            self.len
        );
        ((self.data[index >> ITEM_BITS_LOG2] >> (index & ITEM_BITS_MASK)) & 1) != 0
    }

    /// Push a bit to the end.
    #[inline]
    pub fn push(&mut self, bit: bool) {
        if self.len & ITEM_BITS_MASK == 0 {
            self.data.push(0);
        }
        *self.data.last_mut().unwrap() |= (bit as Storage) << (self.len & ITEM_BITS_MASK);
        self.len += 1;
    }

    /// Extend another bitstring to the end of self.
    pub fn extend(&mut self, another: &BitString) {
        if another.len() == 0 {return}
        if self.len & ITEM_BITS_MASK == 0 {
            self.data.extend_from_slice(&another.data);
            self.len += another.len();
        } else {
            let remaining = ITEM_BITS - (self.len & ITEM_BITS_MASK);
            let last = self.data.len() - 1;
            self.data[last] |=
                (another.data[0] & ((1 << remaining) - 1)) << (self.len & ITEM_BITS_MASK);
            if remaining < another.len() {
                self.len += remaining;
                self.extend(&(another << remaining));
            } else {
                self.len += another.len();
            }
        }
    }

    /// Joins several bitstring
    pub fn join(s: &[BitString]) -> Self {
        let total_length = s.iter().map(|x| x.len()).sum::<usize>();
        let mut ret = Self::with_capacity(total_length);
        for x in s {
            ret.extend(x);
        }
        ret
    }

    /// Split the bitstring to several bitstring of same length. Total length must be a multiple of `length`.
    pub fn split_to_equal_length(&self, length: usize) -> Vec<Self> {
        assert!(
            self.len % length == 0,
            "Total length {} must be a multiple of length {}.",
            self.len,
            length
        );
        let count = self.len / length;
        let mut ret = Vec::with_capacity(count);
        for i in 0..self.len / length {
            ret.push(self.substring(i * length, (i + 1) * length));
        }
        ret
    }

    /// Bit length
    pub fn len(&self) -> usize {
        self.len
    }

    /// Produces a iterator that iterates over the bits.
    pub fn iter(&self) -> BitStringIterator {
        BitStringIterator {
            target: &self,
            index: 0,
        }
    }

    /// Get a byte of the bitstring representing the `8*index` through `8*(1+index)` bits.
    pub fn get_byte(&self, index: usize) -> u8 {
        let index = index * 8;
        ((self.data[index >> ITEM_BITS_LOG2] >> (index & ITEM_BITS_MASK)) & 0xff) as u8
    }

    /// Ceiling division of the bit length by 8.
    pub fn byte_length(&self) -> usize {
        ceil_div(self.len, 8)
    }

    /// Resize the bitstring. If the new length is larger than the old length, the new bits are set to 0.
    pub fn resize(&mut self, new_len: usize) {
        if new_len == self.len {
            return
        } else if new_len > self.len {
            self.data.resize(ceil_div(new_len, ITEM_BITS), 0);
        } else {
            self.data.resize(ceil_div(new_len, ITEM_BITS), 0);
            let last_bits = new_len & ITEM_BITS_MASK;
            if last_bits > 0 {
                let id = self.data.len() - 1;
                self.data[id] &= (1 << last_bits) - 1;
            }
        }
        self.len = new_len;
    }

    /// Reaches the internal data.
    #[inline]
    pub fn data(&self) -> &[Storage] {
        &self.data
    }

    /// Reaches the internal data.
    #[inline]
    pub fn data_mut(&mut self) -> &mut [Storage] {
        &mut self.data
    }

    /// How many ones are there in the bitstring.
    pub fn count_ones(&self) -> usize {
        self.data.iter().map(|x| x.count_ones() as usize).sum()
    }

    /// XOR self with another bitstring. If the other bitstring is longer, self is extended as if with extra 0s.
    #[inline]
    pub fn xor_inplace(&mut self, other: &Self) {
        if other.len() > self.len() {
            self.resize(other.len());
        }
        for i in 0..other.data.len() {
            self.data[i] ^= other.data[i];
        }
    }

    /// XOR one bit.
    #[inline]
    pub fn xor_bit_inplace(&mut self, index: usize, bit: bool) {
        assert!(
            index < self.len,
            "Index {} must be smaller than len {}.",
            index,
            self.len
        );
        if bit {
            self.data[index >> ITEM_BITS_LOG2] ^= (1 as Storage) << (index & ITEM_BITS_MASK);
        }
    }

    /// Invert one bit.
    #[inline]
    pub fn not_bit_inplace(&mut self, index: usize) {
        self.data[index >> ITEM_BITS_LOG2] ^= (1 as Storage) << (index & ITEM_BITS_MASK);
    }

    /// Inverse self inplace.
    #[inline]
    pub fn not_inplace(&mut self) {
        for i in 0..self.data.len() {
            self.data[i] = !self.data[i];
        }
        let last_bits = self.len() & ITEM_BITS_MASK;
        if last_bits > 0 {
            let id = self.data.len() - 1;
            self.data[id] &= (1 << last_bits) - 1;
        }
    }

    /// Get the last place where a 1 occurs. If no 1 occurs, return None.
    pub fn last_one_index(&self) -> Option<usize> {
        let mut id = self.data.len() - 1;
        while id > 0 && self.data[id] == 0 {
            id -= 1;
        }
        if self.data[id] == 0 {
            return None;
        }
        Some((id + 1) * ITEM_BITS - self.data[id].leading_zeros() as usize - 1)
    }

    /// Split self into halves. Panic if the bitlength is not a multiple of 2.
    pub fn take_halves(mut self) -> (BitString, BitString) {
        if self.len() & 1 != 0 {
            panic!("Cannot split for odd length.");
        }
        let c = self.len() / 2;
        if c & ITEM_BITS_MASK == 0 {
            let rdata = self.data.split_off(c >> ITEM_BITS_LOG2);
            let right = Self {
                data: rdata,
                len: c,
            };
            let ldata = self.data;
            let left = Self {
                data: ldata,
                len: c,
            };
            (left, right)
        } else {
            let right = &self << c;
            self.resize(c);
            (self, right)
        }
    }

    /// Split self into halves, where the first half is guaranteed to have length of a power of 2. The other half is what remains.
    /// For example, 37 bits -> (32 + 5) bits, 16 bits -> (8 + 8) bits.
    pub fn two_power_halves(&self) -> (BitString, BitString) {
        if self.len() <= 1 {
            panic!("Cannot split for length < 1");
        }
        let logc = log2ceil(self.len()) - 1;
        let c = 1 << logc;
        if self.data.len() > 1 {
            let c = c / ITEM_BITS;
            let left = Self {
                data: self.data[..c].to_vec(),
                len: c * ITEM_BITS,
            };
            let mut right = Self {
                data: self.data[c..].to_vec(),
                len: self.len - c * ITEM_BITS,
            };
            right.resize(c * ITEM_BITS);
            (left, right)
        } else {
            let p = self.data[0];
            let left = p & ((1 << c) - 1);
            let right = p >> c;
            (
                Self {
                    data: vec![left],
                    len: c,
                },
                Self {
                    data: vec![right],
                    len: c,
                },
            )
        }
    }

    /// Takes a substring. Panics if end > start.
    pub fn substring(&self, start: usize, end: usize) -> Self {
        assert!(start <= end, "Start must be smaller than end.");
        assert!(end <= self.len, "End must be smaller than length.");
        let total_bits = end - start;
        let total_chunks = ceil_div(total_bits, ITEM_BITS);
        let start_chunk_index = start >> ITEM_BITS_LOG2;
        let data = if start & ITEM_BITS_MASK == 0 {
            self.data[start_chunk_index..(start_chunk_index+total_chunks)].to_vec()
        } else {
            let part_bits = start & ITEM_BITS_MASK;
            (0..total_chunks).map(|i| {
                let combined = (self.data[start_chunk_index + i] >> part_bits) | 
                (if start_chunk_index + i + 1 < self.data.len() {self.data[start_chunk_index + i + 1] << (ITEM_BITS - part_bits)} else {0});
                combined
            }).collect()
        };
        let mut ret = Self {
            data,
            len: total_bits,
        };
        ret.ensure_last_chunk();
        ret
    }

    /// Split the bitstring at `at`, and the `self` is left with the first part, returning the second part.
    pub fn split_off(&mut self, at: usize) -> Self {
        let taken = self.substring(at, self.len());
        self.resize(at);
        taken
    }

    /// Split the bitstring into `count` chunks, but guarantee each chunk's length is a multiple of 64.
    pub fn split_uniform(&self, count: usize) -> Vec<Self> {
        if count == 1 {
            vec![self.clone()]
        } else {
            let interval = ceil_div(self.len(), count);
            let interval = ceil_div(interval, 64) * 64;
            (0..count).map(|i| {
                let start = self.len().min(i * interval);
                let end = self.len().min((i+1) * interval);
                self.substring(start, end)
            }).collect::<Vec<_>>()
        }
    }

    /// Merge a vector of bitstrings into one.
    pub fn merge(x: Vec<BitString>) -> Self {
        if x.len() == 1 {
            return x.into_iter().next().unwrap();
        }
        let total_length = x.iter().map(|x| x.len()).sum::<usize>();
        let mut ret = Self::with_capacity(total_length);
        for item in x {
            ret.extend(&item);
        }
        ret
    }

    /// Dot product of two bitstrings, first bit-and then xor all. Lengths must match.
    /// For example, (1011) dot (1110) = 0.
    pub fn dot(&self, other: &Self) -> bool {
        assert!(self.len() == other.len(), "Lengths must be equal.");
        let mut ret = false;
        for i in 0..self.data.len() {
            ret ^= (self.data[i] & other.data[i]).count_ones() % 2 != 0;
        }
        ret
    }

    /// Get the last bit, and self shrinks by 1 bit.
    pub fn pop(&mut self) -> bool {
        assert!(self.len() > 0, "Length = 0. Have nothing to pop.");
        let bit = self.get(self.len() - 1);
        self.set(self.len() - 1, false);
        self.len -= 1;
        bit
    }

    /// Set a range to 0 or 1.
    pub fn set_range(&mut self, start: usize, end: usize, bit: bool) {
        assert!(start <= end);
        let block_start = (start + ITEM_BITS - 1) >> ITEM_BITS_LOG2;
        let block_end = end >> ITEM_BITS_LOG2;
        if block_start <= block_end {
            for i in block_start..block_end {
                self.data[i] = if bit { !0 } else { 0 };
            }
            if bit {
                if (start & ITEM_BITS_MASK) > 0 {
                    self.data[block_start - 1] |= !((1 << (start & ITEM_BITS_MASK)) - 1);
                }
                if (end & ITEM_BITS_MASK) > 0 {
                    self.data[block_end] |= (1 << (end & ITEM_BITS_MASK)) - 1;
                }
            } else {
                if (start & ITEM_BITS_MASK) > 0 {
                    self.data[block_start - 1] &= (1 << (start & ITEM_BITS_MASK)) - 1;
                }
                if (end & ITEM_BITS_MASK) > 0 {
                    self.data[block_end] &= !((1 << (end & ITEM_BITS_MASK)) - 1);
                }
            }
        } else {
            if bit {
                self.data[block_start - 1] |=
                    ((1 << (end - start)) - 1) << (start & ITEM_BITS_MASK);
            } else {
                self.data[block_start - 1] &=
                    !(((1 << (end - start)) - 1) << (start & ITEM_BITS_MASK));
            }
        }
    }

    /// Get the number of consecutive zeros from the beginning
    pub fn leading_zeros(&self) -> usize {
        let mut ret = 0;
        for i in 0..self.data.len() {
            if self.data[i] == 0 {
                ret += ITEM_BITS;
            } else {
                ret += self.data[i].trailing_zeros() as usize;
                break;
            }
        }
        ret
    }
}

impl Default for BitString {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for BitString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.len {
            std::fmt::Write::write_char(f, if self.get(i) { '1' } else { '0' })?
        }
        Ok(())
    }
}

impl From<&[u8]> for BitString {
    fn from(s: &[u8]) -> Self {
        let len = ceil_div(s.len() * 8, ITEM_BITS);
        let bottom_len = (s.len() * 8) >> ITEM_BITS_LOG2;
        unsafe {
            let ptr = s.as_ptr() as *const Storage;
            let mut data = vec![0; len];
            for i in 0..bottom_len {
                data[i] = *ptr.add(i);
            }
            for i in (bottom_len * ITEM_BITS >> 3)..s.len() {
                let byte = *s.get_unchecked(i);
                data[len - 1] |= (byte as Storage) << ((i << 3) & ITEM_BITS_MASK);
            }
            Self {
                data,
                len: s.len() * 8,
            }
        }
    }
}

impl From<&[usize]> for BitString {
    fn from(s: &[usize]) -> Self {
        Self {
            data: s.to_vec(),
            len: s.len() * std::mem::size_of::<usize>() * 8,
        }
    }
}

impl From<Vec<u8>> for BitString {
    fn from(s: Vec<u8>) -> Self {
        Self::from(&s[..])
    }
}

impl From<&Vec<u8>> for BitString {
    fn from(s: &Vec<u8>) -> Self {
        Self::from(&s[..])
    }
}

impl From<Vec<usize>> for BitString {
    fn from(s: Vec<usize>) -> Self {
        Self::from(&s[..])
    }
}

impl From<&Vec<usize>> for BitString {
    fn from(s: &Vec<usize>) -> Self {
        Self::from(&s[..])
    }
}

impl From<bool> for BitString {
    fn from(b: bool) -> Self {
        let mut ret = Self::new_zeros(1);
        ret.set(0, b);
        ret
    }
}

impl From<&[bool]> for BitString {
    fn from(s: &[bool]) -> Self {
        let mut ret = Self::new_zeros(s.len());
        for i in 0..s.len() {
            ret.set(i, s[i]);
        }
        ret
    }
}

impl From<Vec<bool>> for BitString {
    fn from(s: Vec<bool>) -> Self {
        Self::from(&s[..])
    }
}

impl From<&Vec<bool>> for BitString {
    fn from(s: &Vec<bool>) -> Self {
        Self::from(&s[..])
    }
}

impl From<&BitString> for [u64; 2] {
    #[cfg(target_pointer_width = "64")]
    fn from(s: &BitString) -> [u64; 2] {
        assert_eq!(s.len(), 128);
        let mut ret = [0; 2];
        ret[0] = s.data[0] as u64;
        ret[1] = s.data[1] as u64;
        ret
    }

    #[cfg(target_pointer_width = "32")]
    fn from(s: &BitString) -> [u64; 2] {
        assert_eq!(s.len(), 128);
        let mut ret = [0; 2];
        ret[0] = (s.data[0] as u32) | ((s.data[1] as u32) << 32);
        ret[1] = (s.data[2] as u32) | ((s.data[3] as u32) << 32);
        ret
    }
}

impl From<&BitString> for [u8; 16] {
    #[cfg(target_pointer_width = "64")]
    fn from(s: &BitString) -> [u8; 16] {
        assert_eq!(s.len(), 128);
        let mut ret = [0; 16];
        unsafe {
            let ptr = ret.as_mut_ptr() as *mut u64;
            *ptr.add(0) = s.data[0] as u64;
            *ptr.add(1) = s.data[1] as u64;
        }
        ret
    }

    #[cfg(target_pointer_width = "32")]
    fn from(s: &BitString) -> [u8; 16] {
        assert_eq!(s.len(), 128);
        let mut ret = [0; 16];
        unsafe {
            let ptr = ret.as_mut_ptr() as *mut u32;
            *ptr.add(0) = s.data[0] as u32;
            *ptr.add(1) = s.data[1] as u32;
            *ptr.add(2) = s.data[2] as u32;
            *ptr.add(3) = s.data[3] as u32;
        }
        ret
    }
}

impl From<&BitString> for Vec<u8> {
    #[cfg(target_pointer_width = "64")]
    fn from(s: &BitString) -> Vec<u8> {
        let mut count = ceil_div(s.len(), 8);
        let mut ret = Vec::with_capacity(count);
        for item in s.data() {
            let mut item = *item;
            for _ in 0..8 {
                ret.push(item as u8);
                item >>= 8;
                count -= 1;
                if count == 0 {
                    break;
                }
            }
        }
        ret
    }

    #[cfg(target_pointer_width = "32")]
    fn from(s: &BitString) -> Vec<u8> {
        let mut count = ceil_div(s.len(), 8);
        let mut ret = Vec::with_capacity(count);
        for item in s.data() {
            let mut item = *item;
            for _ in 0..4 {
                ret.push(item as u8);
                item >>= 8;
                count -= 1;
                if count == 0 {
                    break;
                }
            }
        }
        ret
    }
}

impl From<&BitString> for Vec<u128> {
    #[cfg(target_pointer_width = "64")]
    fn from(s: &BitString) -> Vec<u128> {
        let count = ceil_div(s.len(), 128);
        let mut ret = vec![0; count];
        for (i, item) in s.data().iter().enumerate() {
            ret[i / 2] |= (*item as u128) << ((i & 1) * 64);
        }
        ret
    }

    #[cfg(target_pointer_width = "32")]
    fn from(s: &BitString) -> Vec<u128> {
        let count = ceil_div(s.len(), 128);
        let mut ret = vec![0; count];
        for (i, item) in s.data().iter().enumerate() {
            ret[i / 4] |= (*item as u128) << ((i & 3) * 32);
        }
        ret
    }
}

impl From<&BitString> for Vec<Block> {
    fn from(s: &BitString) -> Vec<Block> {
        let r = <Vec<u128>>::from(s);
        r.into_iter().map(|x| x.into()).collect()
    }
}

/// Iterator for BitString
pub struct BitStringIterator<'a> {
    target: &'a BitString,
    index: usize,
}

impl<'a> Iterator for BitStringIterator<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.target.len() > self.index {
            let ret = Some(self.target.get(self.index));
            self.index += 1;
            ret
        } else {
            None
        }
    }
}

impl std::ops::BitXor<&BitString> for &BitString {
    type Output = BitString;
    #[inline]
    fn bitxor(self, rhs: &BitString) -> Self::Output {
        assert_eq!(
            self.len(),
            rhs.len(),
            "The Xor'ed bit strings have different length."
        );
        let mut ret = self.clone();
        ret.xor_inplace(rhs);
        ret
    }
}
impl std::ops::BitXor<&BitString> for BitString {
    type Output = BitString;
    #[inline]
    fn bitxor(self, rhs: &BitString) -> Self::Output {
        &self ^ rhs
    }
}
impl std::ops::BitXor<BitString> for &BitString {
    type Output = BitString;
    #[inline]
    fn bitxor(self, rhs: BitString) -> Self::Output {
        self ^ &rhs
    }
}
impl std::ops::BitXor<BitString> for BitString {
    type Output = BitString;
    #[inline]
    fn bitxor(self, rhs: BitString) -> Self::Output {
        &self ^ &rhs
    }
}
impl std::ops::BitXorAssign<&BitString> for BitString {
    #[inline]
    fn bitxor_assign(&mut self, rhs: &BitString) {
        assert_eq!(
            self.len(),
            rhs.len(),
            "The Xor'ed bit strings have different length."
        );
        self.xor_inplace(rhs);
    }
}
impl std::ops::BitXorAssign<BitString> for BitString {
    #[inline]
    fn bitxor_assign(&mut self, rhs: BitString) {
        *self ^= &rhs;
    }
}

impl std::ops::BitAnd<&BitString> for &BitString {
    type Output = BitString;
    #[inline]
    fn bitand(self, rhs: &BitString) -> Self::Output {
        assert_eq!(
            self.len(),
            rhs.len(),
            "The Xor'ed bit strings have different length."
        );
        let out_data = self
            .data
            .iter()
            .zip(rhs.data.iter())
            .map(|(x, y)| (*x) & (*y))
            .collect::<Vec<_>>();
        Self::Output {
            data: out_data,
            len: rhs.len(),
        }
    }
}
impl std::ops::BitAnd<&BitString> for BitString {
    type Output = BitString;
    #[inline]
    fn bitand(self, rhs: &BitString) -> Self::Output {
        &self & rhs
    }
}
impl std::ops::BitAnd<BitString> for &BitString {
    type Output = BitString;
    #[inline]
    fn bitand(self, rhs: BitString) -> Self::Output {
        self & &rhs
    }
}
impl std::ops::BitAnd<BitString> for BitString {
    type Output = BitString;
    #[inline]
    fn bitand(self, rhs: BitString) -> Self::Output {
        &self & &rhs
    }
}

impl std::ops::BitOr<&BitString> for &BitString {
    type Output = BitString;

    fn bitor(self, rhs: &BitString) -> Self::Output {
        assert_eq!(
            self.len(),
            rhs.len(),
            "The Xor'ed bit strings have different length."
        );
        let out_data = self
            .data
            .iter()
            .zip(rhs.data.iter())
            .map(|(x, y)| (*x) | (*y))
            .collect::<Vec<_>>();
        Self::Output {
            data: out_data,
            len: rhs.len(),
        }
    }
}
impl std::ops::BitOr<&BitString> for BitString {
    type Output = BitString;
    #[inline]
    fn bitor(self, rhs: &BitString) -> Self::Output {
        &self | rhs
    }
}
impl std::ops::BitOr<BitString> for &BitString {
    type Output = BitString;
    #[inline]
    fn bitor(self, rhs: BitString) -> Self::Output {
        self | &rhs
    }
}
impl std::ops::BitOr<BitString> for BitString {
    type Output = BitString;
    #[inline]
    fn bitor(self, rhs: BitString) -> Self::Output {
        &self | &rhs
    }
}
impl std::ops::Not for &BitString {
    type Output = BitString;
    fn not(self) -> Self::Output {
        let mut ret = self.clone();
        ret.not_inplace();
        ret
    }
}
impl std::ops::Not for BitString {
    type Output = BitString;
    fn not(self) -> Self::Output {
        !&self
    }
}

impl std::ops::Shl<usize> for &BitString {
    type Output = BitString;
    fn shl(self, rhs: usize) -> Self::Output {
        let span = rhs / ITEM_BITS;
        let shift = rhs % ITEM_BITS;
        let result_length = self.len() - rhs;
        let mut result = vec![0; ceil_div(result_length, ITEM_BITS)];
        for i in 0..result.len() {
            let mut item = 0;
            item = item | (self.data[i + span].wrapping_shr(shift as u32));
            if shift != 0 && i + span + 1 < self.data.len() {
                item =
                    item | ((self.data[i + span + 1] & ((1 << shift) - 1)) << (ITEM_BITS - shift));
            }
            result[i] = item;
        }
        BitString {
            data: result,
            len: result_length,
        }
    }
}
impl std::ops::Shl<usize> for BitString {
    type Output = BitString;

    fn shl(self, rhs: usize) -> Self::Output {
        &self << rhs
    }
}

impl std::ops::Shr<usize> for &BitString {
    type Output = BitString;
    fn shr(self, rhs: usize) -> Self::Output {
        let span = rhs / ITEM_BITS;
        let shift = rhs % ITEM_BITS;
        let result_length = self.len() + rhs;
        let mut result = vec![0; ceil_div(result_length, ITEM_BITS)];
        for i in 0..result.len() {
            let mut item = 0;
            if shift != 0 && i >= span + 1 && i - span - 1 < self.data.len() {
                item = item | (self.data[i - span - 1].wrapping_shr((ITEM_BITS - shift) as u32));
            }
            if i >= span && i - span < self.data.len() {
                item = item | (self.data[i - span] << shift);
            }
            result[i] = item;
        }
        BitString {
            data: result,
            len: result_length,
        }
    }
}
impl std::ops::Shr<usize> for BitString {
    type Output = BitString;

    fn shr(self, rhs: usize) -> Self::Output {
        &self >> rhs
    }
}

impl Hashable for BitString {
    fn append_to_hasher(&self, hasher: &mut blake3::Hasher) {
        hasher.update(&<Vec<u8>>::from(self));
    }
}

impl Hashable for &BitString {
    fn append_to_hasher(&self, hasher: &mut blake3::Hasher) {
        hasher.update(&<Vec<u8>>::from(*self));
    }
}

impl PartialEq for BitString {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.data == other.data
    }
}

impl std::fmt::Debug for BitString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use the Display implementation
        write!(f, "BitString {}", self)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    pub fn set() {
        let mut x = BitString::new_zeros(128);
        assert_eq!(format!("{}", x), "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
        x.set(4, true);
        assert_eq!(format!("{}", x), "00001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
        x.set(13, true);
        assert_eq!(format!("{}", x), "00001000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
        x.set(4, false);
        assert_eq!(format!("{}", x), "00000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
        assert_eq!(x.get(13), true);
        assert_eq!(x.get(14), false);
    }

    #[test]
    pub fn xor() {
        let test = |len1, len2| {
            let x1 = BitString::new_random(len1);
            let x2 = BitString::new_random(len2);
            let mut x3 = x1.clone();
            x3.xor_inplace(&x2);
            let min = std::cmp::min(len1, len2);
            for i in 0..min {
                assert_eq!(x3.get(i), x1.get(i) ^ x2.get(i));
            }
            for i in min..len1 {
                assert_eq!(x3.get(i), x1.get(i));
            }
            for i in min..len2 {
                assert_eq!(x3.get(i), x2.get(i));
            }
        };
        test(0, 0);
        test(0, 1);
        test(1, 0);
        test(1, 1);
        test(1, 10);
        test(10, 1);
        test(10, 10);
        test(10, 100);
        test(100, 10);
    }

    #[test]
    pub fn shift_left() {
        let x = BitString::from(vec![0x0123456789abcdefusize, 0xfedcba9876543210usize]);
        let y = &x << 4;
        assert_eq!(
            y.data(),
            &[0x00123456789abcdeusize, 0x0fedcba987654321usize]
        );
        let y = &x << 68;
        assert_eq!(y.data(), &[0x0fedcba987654321usize]);
        let test = |len, shift| {
            let x = BitString::new_random(len);
            let y = &x << shift;
            for i in 0..len - shift {
                assert_eq!(x.get(i + shift), y.get(i));
            }
            assert_eq!(y.len(), len - shift);
        };
        test(1, 1);
        test(10, 2);
        test(100, 3);
        test(200, 5);
        test(200, 64);
        test(200, 73);
    }

    #[test]
    pub fn shift_right() {
        let x = BitString::from(vec![0x0123456789abcdefusize, 0xfedcba9876543210usize]);
        assert_eq!(
            x.data(),
            &[0x0123456789abcdefusize, 0xfedcba9876543210usize]
        );
        let y = &x >> 4;
        assert_eq!(
            y.data(),
            &[0x123456789abcdef0usize, 0xedcba98765432100usize, 0xf]
        );
        let y = &x >> 68;
        assert_eq!(
            y.data(),
            &[0x0, 0x123456789abcdef0usize, 0xedcba98765432100usize, 0xf]
        );
    }

    #[test]
    pub fn resize() {
        let x = BitString::from(vec![0x0123456789abcdefusize, 0xfedcba9876543210usize]);
        let mut y = x.clone();
        y.resize(132);
        assert_eq!(
            y.data(),
            &[0x0123456789abcdefusize, 0xfedcba9876543210usize, 0x0]
        );
        y.resize(120);
        assert_eq!(
            y.data(),
            &[0x0123456789abcdefusize, 0x00dcba9876543210usize]
        );
        y.resize(56);
        assert_eq!(y.data(), &[0x0023456789abcdefusize]);
        let test = |len, shift| {
            let x = BitString::new_random(len);
            let y = &x >> shift;
            for i in 0..len {
                assert_eq!(x.get(i), y.get(i + shift));
            }
            for i in 0..shift {
                assert_eq!(y.get(i), false);
            }
            assert_eq!(y.len(), len + shift);
        };
        test(1, 1);
        test(10, 2);
        test(100, 3);
        test(200, 5);
        test(200, 64);
        test(200, 73);
    }

    #[test]
    pub fn last_one_index() {
        let x = BitString::from(vec![0b0u8]);
        assert_eq!(x.last_one_index(), None);
        let x = BitString::from(vec![0b1u8]);
        assert_eq!(x.last_one_index(), Some(0));
        let x = BitString::from(vec![0b10u8]);
        assert_eq!(x.last_one_index(), Some(1));
    }

    #[test]
    pub fn two_power_halves() {
        let mut x = BitString::from(vec![0b10u8]);
        x.resize(2);
        assert_eq!(format!("{}", x), "01");
        let (l, r) = x.two_power_halves();
        assert_eq!(format!("{}", l), "0");
        assert_eq!(format!("{}", r), "1");
        let mut x = BitString::from(vec![0b100u8]);
        x.resize(3);
        let (l, r) = x.two_power_halves();
        assert_eq!(format!("{}", l), "00");
        assert_eq!(format!("{}", r), "10");
        let mut x = BitString::from(vec![0b10010110011usize]);
        x.resize(11);
        let (l, r) = x.two_power_halves();
        assert_eq!(format!("{}", l), "11001101");
        assert_eq!(format!("{}", r), "00100000");
        let x = BitString::from(vec![0x0123456789abcdefusize, 0xfedcba9876543210usize]);
        let (l, r) = x.two_power_halves();
        assert_eq!(l.data(), &[0x0123456789abcdefusize]);
        assert_eq!(r.data(), &[0xfedcba9876543210usize]);
        let x = BitString::from(vec![
            0x0123456789abcdefusize,
            0xfedcba9876543210usize,
            0x0123usize,
        ]);
        let (l, r) = x.two_power_halves();
        assert_eq!(
            l.data(),
            &[0x0123456789abcdefusize, 0xfedcba9876543210usize]
        );
        assert_eq!(r.data(), &[0x0123usize, 0]);
    }

    #[test]
    pub fn test_extend() {
        let mut x = BitString::from(vec![0u8]);
        x.resize(64);
        let mut y = BitString::from(vec![0b10u8]);
        y.resize(2);
        x.extend(&y);
        assert_eq!(
            format!("{}", x),
            "000000000000000000000000000000000000000000000000000000000000000001"
        );
        let mut x = BitString::from(vec![0u8]);
        x.resize(63);
        let mut y = BitString::from(vec![0b11u8]);
        y.resize(2);
        x.extend(&y);
        assert_eq!(
            format!("{}", x),
            "00000000000000000000000000000000000000000000000000000000000000011"
        );
    }

    #[test]
    pub fn test_set_range() {
        let mut x = BitString::new();
        x.resize(129);
        assert_eq!(format!("{}", x), "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
        x.set_range(2, 5, true);
        assert_eq!(format!("{}", x), "001110000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
        x.set_range(3, 5, false);
        assert_eq!(format!("{}", x), "001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
        x.set_range(0, 67, true);
        assert_eq!(format!("{}", x), "111111111111111111111111111111111111111111111111111111111111111111100000000000000000000000000000000000000000000000000000000000000");
        x.set_range(3, 129, false);
        assert_eq!(format!("{}", x), "111000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    pub fn test_substring() {
        let naive_substring = |x: &BitString, start: usize, end: usize| {
            let mut y = BitString::new_zeros(end - start);
            for i in start..end {
                y.set(i - start, x.get(i))
            }
            y
        };
        let test = |len, start, end| {
            let x = BitString::new_random(len);
            let y = x.substring(start, end);
            assert_eq!(y, naive_substring(&x, start, end));
        };
        test(1, 0, 1);
        test(10, 2, 5);
        test(100, 3, 10);
        test(200, 64, 128);
        test(200, 64, 150);
        test(200, 63, 150);
        test(200, 65, 150);
    }

    #[test]
    pub fn test_constructors() {
        let test_zeros_ones = |zeros: usize, ones: usize| {
            let x = BitString::new_zeros_ones(zeros, ones);
            assert_eq!(x.len(), zeros + ones);
            for i in 0..zeros {assert_eq!(x.get(i), false, "zeros {} ones {} at {}", zeros, ones, i);}
            for i in 0..ones {assert_eq!(x.get(i+zeros), true, "zeros {} ones {} at {}", zeros, ones, zeros+i);}
        };
        test_zeros_ones(0, 0);
        test_zeros_ones(0, 1);
        test_zeros_ones(1, 0);
        test_zeros_ones(1, 1);
        test_zeros_ones(64, 128);
        test_zeros_ones(64, 150);
        test_zeros_ones(63, 128);
        test_zeros_ones(65, 128);

        let test_ones_zeros = |ones: usize, zeros: usize| {
            let x = BitString::new_ones_zeros(ones, zeros);
            assert_eq!(x.len(), zeros + ones);
            for i in 0..ones {assert_eq!(x.get(i), true, "ones {} zeros {} at {}", ones, zeros, i);}
            for i in 0..zeros {assert_eq!(x.get(i+ones), false, "ones {} zeros {} at {}", ones, zeros, ones+i);}
        };
        test_ones_zeros(0, 0);
        test_ones_zeros(0, 1);
        test_ones_zeros(1, 0);
        test_ones_zeros(1, 1);
        test_ones_zeros(64, 128);
        test_ones_zeros(64, 150);
        test_ones_zeros(63, 128);
        test_ones_zeros(65, 128);
    }
}

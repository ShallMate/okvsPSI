//! Provides Hashing and PRNG utilities.

pub use blake3::Hasher;
use crate::aes::fixed_aes_hash_single;
use crate::Block;
use crate::bitstring::BitString;
use crate::aes::fixed_aes_hash;

/// Produce a hash of the given length from the given hasher.
pub fn hash_to_length(hasher: Hasher, length: usize) -> Vec<u8> {
    let mut ret = vec![0; length];
    hasher.finalize_xof().fill(&mut ret);
    ret
}

/// Functions that allow an object to be hashed to specified output objects.
pub trait Hashable where Self: Sized {
    /// Append the information of Self to an existing hasher. Note that the order of appending matters.
    fn append_to_hasher(&self, hasher: &mut Hasher);
    /// Hash the information of Self to a new hasher.
    #[inline]
    fn hash_to_hasher(&self) -> Hasher {
        let mut hasher = Hasher::new();
        self.append_to_hasher(&mut hasher);
        hasher
    }
    /// Hash the information of Self to a vector of the given length.
    #[inline]
    fn hash_to_bytes(&self, length: usize) -> Vec<u8> {
        let hasher = self.hash_to_hasher();
        hash_to_length(hasher, length)
    }
    /// Hash the information of Self to fixed lengthed bytes array.
    #[inline]
    fn hash_to_cbytes<const C: usize>(&self) -> [u8; C] {
        let mut ret = [0u8; C];
        let hasher = self.hash_to_hasher();
        hasher.finalize_xof().fill(&mut ret);
        ret
    }
    /// Hash the information of Self to a bitstring of the given length.
    fn hash_to_bitstring(&self, bit_length: usize) -> BitString {
        let byte_length = crate::utils::ceil_div(bit_length, 8);
        let mut ret = BitString::from(self.hash_to_bytes(byte_length).as_slice());
        ret.resize(bit_length);
        ret
    }
    /// Create a random buffered generator from the information of Self.
    #[inline]
    fn to_buffered_random_generator(&self) -> BufferedRandomGenerator {
        BufferedRandomGenerator::new(self.hash_to_block())
    }
    /// To a block
    #[inline]
    fn hash_to_block(&self) -> crate::Block {
        let mut hasher = self.hash_to_hasher().finalize_xof();
        let mut block = Block::default();
        unsafe {
            let ptr = std::slice::from_raw_parts_mut((&mut block) as *mut Block as *mut u8, 16);
            hasher.fill(ptr);
        }
        block
    }
    /// Hash the information of Self to a bool.
    #[inline]
    fn hash_to_bool(&self) -> bool {
        let r = self.hash_to_hasher();
        r.finalize().as_bytes()[0] % 2 == 1
    }
    /// Hash the information of Self to a random generator.
    #[inline]
    fn to_random_generator(&self) -> RandomGenerator {
        let hasher = self.hash_to_hasher();
        RandomGenerator::from_raw_parts(hasher.finalize_xof())
    }
}

impl Hashable for Vec<u8> {
    #[inline]
    fn append_to_hasher(&self, hasher: &mut Hasher) {
        hasher.update(self);
    }
}

impl Hashable for &Vec<u8> {
    #[inline]
    fn append_to_hasher(&self, hasher: &mut Hasher) {
        hasher.update(self);
    }
}

impl Hashable for &[u8] {
    #[inline]
    fn append_to_hasher(&self, hasher: &mut Hasher) {
        hasher.update(self);
    }
}

impl Hashable for bool {
    #[inline]
    fn append_to_hasher(&self, hasher: &mut Hasher) {
        hasher.update(&[*self as u8]);
    }
}

impl Hashable for usize {
    #[inline]
    fn append_to_hasher(&self, hasher: &mut Hasher) {
        hasher.update(&self.to_le_bytes());
    }
}

impl Hashable for u64 {
    #[inline]
    fn append_to_hasher(&self, hasher: &mut Hasher) {
        hasher.update(&self.to_le_bytes());
    }
}

impl Hashable for u128 {
    #[inline]
    fn append_to_hasher(&self, hasher: &mut Hasher) {
        hasher.update(&self.to_le_bytes());
    }
}

impl Hashable for &u128 {
    #[inline]
    fn append_to_hasher(&self, hasher: &mut Hasher) {
        hasher.update(&self.to_le_bytes());
    }
}

impl Hashable for &Block {
    #[inline]
    fn append_to_hasher(&self, hasher: &mut Hasher) {
        self.0.append_to_hasher(hasher);
    }
    #[inline]
    fn hash_to_block(&self) -> crate::Block {
        crate::aes::hash_block_to_block(self)
    }
}

impl Hashable for Block {
    #[inline]
    fn append_to_hasher(&self, hasher: &mut Hasher) {
        (&self).append_to_hasher(hasher);
    }
    #[inline]
    fn hash_to_bytes(&self, length: usize) -> Vec<u8> {
        if length > 16 {
            let mut ret = vec![0; length];
            let mut hasher = self.hash_to_hasher().finalize_xof();
            hasher.fill(&mut ret);
            ret
        } else {
            let mut out = Block::default();
            crate::aes::fixed_aes_hash_single(self, &mut out);
            out.0.to_le_bytes()[..length].to_vec()
        }   
    }
    #[inline]
    fn hash_to_block(&self) -> crate::Block {
        crate::aes::hash_block_to_block(self)
    }
    #[inline]
    fn hash_to_bool(&self) -> bool {
        let r = self.hash_to_block();
        r.0 & 1 != 0
    }
    
}

impl Hashable for u8 {
    #[inline]
    fn append_to_hasher(&self, hasher: &mut Hasher) {
        hasher.update(&[*self]);
    }
}

impl<T1, T2> Hashable for (T1, T2)
where
    T1: Hashable,
    T2: Hashable,
{
    fn append_to_hasher(&self, hasher: &mut Hasher) {
        self.0.append_to_hasher(hasher);
        self.1.append_to_hasher(hasher);
    }
}

impl Hashable for &[u64; 2] {
    fn append_to_hasher(&self, hasher: &mut Hasher) {
        hasher.update(&self[0].to_le_bytes());
        hasher.update(&self[1].to_le_bytes());
    }
}

/// Hash to a specified type.
pub trait HashTo<T> {
    /// Hash self to T
    fn hash_to(&self) -> T;
    /// Hash `Vec<self>` to `Vec<T>`
    fn hash_vec_to(input: &[Self]) -> Vec<T> where Self: Sized {
        input.iter().map(|each| each.hash_to()).collect()
    }
    /// Hash `Vec<(self, self)>` to `Vec<(T, T)>`
    fn hash_pair_vec_to(input: &[(Self, Self)]) -> Vec<(T, T)> where Self: Sized {
        input.iter().map(|each| (each.0.hash_to(), each.1.hash_to())).collect()
    }
}

impl<T> HashTo<BufferedRandomGenerator> for T where T: Hashable {
    #[inline] fn hash_to(&self) -> BufferedRandomGenerator {
        self.to_buffered_random_generator()
    }
}

impl<T> HashTo<Block> for T where T: Hashable + Clone + std::any::Any {
    #[inline] fn hash_to(&self) -> Block {
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<Block>() {
            let mut out = Block::default();
            unsafe { fixed_aes_hash_single(&*(self as *const T as *const Block), &mut out); }
            out
        } else {
            self.hash_to_block()
        }
    }
    fn hash_vec_to(input: &[Self]) -> Vec<Block> where Self: Sized {
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<Block>() {
            unsafe {
                let slice = std::slice::from_raw_parts(input.as_ptr() as *const Block, input.len());
                let mut ret = vec![Block::default(); slice.len()];
                fixed_aes_hash(slice, &mut ret);
                // transmute and return
                let ret = std::mem::transmute(ret);
                ret
            }
        } else {
            input.iter().map(|each| each.hash_to()).collect()
        }
    }
    fn hash_pair_vec_to(input: &[(Self, Self)]) -> Vec<(Block, Block)> where Self: Sized {
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<Block>() {
            let mut cloned = input.to_vec();
            unsafe {
                let slice = std::slice::from_raw_parts_mut(cloned.as_mut_ptr() as *mut Block, cloned.len() * 2);
                let mut ret = vec![<(Block, Block)>::default(); input.len()];
                // Note: if we directly define ret as Vec<Block> of length n*2, and use &mut ret for fixed_aes_hash
                // then the transmute would be incorrect as the final result would have n*2 length, but only n pairs.
                // This is because transmute directly interprets (copies) the vector size.
                let out_slice = std::slice::from_raw_parts_mut(ret.as_mut_ptr() as *mut (Block, Block) as *mut Block, slice.len());
                fixed_aes_hash(slice, out_slice);
                // transmute and return
                std::mem::transmute(ret)
            }
        } else {
            input.iter().map(|each| (each.0.hash_to(), each.1.hash_to())).collect()
        }
    }
}

impl HashTo<Block> for Hasher {
    #[inline] fn hash_to(&self) -> Block {
        let mut hasher = self.finalize_xof();
        let mut block = Block::default();
        unsafe {
            let ptr = std::slice::from_raw_parts_mut((&mut block) as *mut Block as *mut u8, 16);
            hasher.fill(ptr);
        }
        block
    }
}

impl<T> HashTo<bool> for T where T: Hashable {
    #[inline] fn hash_to(&self) -> bool {
        self.hash_to_bool()
    }
}

impl HashTo<u64> for Block {
    #[inline] fn hash_to(&self) -> u64 {
        <Block as HashTo<Block>>::hash_to(self).0 as u64
    }
    #[inline] fn hash_vec_to(input: &[Self]) -> Vec<u64> where Self: Sized {
        <Block as HashTo<Block>>::hash_vec_to(input).iter().map(|each| each.0 as u64).collect()
    }
    #[inline] fn hash_pair_vec_to(input: &[(Self, Self)]) -> Vec<(u64, u64)> where Self: Sized {
        <Block as HashTo<Block>>::hash_pair_vec_to(input).iter().map(|each| (each.0.0 as u64, each.1.0 as u64)).collect()
    }
}

/// Hash to an object with specified length.
pub trait HashToLengthed<T> {
    /// Hash self to T with specified length.
    fn hash_to_length(&self, len: usize) -> T;
    /// Hash `Vec<self>` to `Vec<T>` with specified length.
    fn hash_vec_to_length(input: &[Self], len: usize) -> Vec<T> where Self: Sized {
        input.iter().map(|each| each.hash_to_length(len)).collect()
    }
    /// Hash `Vec<(self, self)>` to `Vec<(T, T)>` with specified length.
    fn hash_pair_vec_to_length(input: &[(Self, Self)], len: usize) -> Vec<(T, T)> where Self: Sized {
        input.iter().map(|each| (each.0.hash_to_length(len), each.1.hash_to_length(len))).collect()
    }
}

impl<T> HashToLengthed<Vec<u8>> for T where T: Hashable + std::any::Any {
    #[inline] fn hash_to_length(&self, len: usize) -> Vec<u8> {
        self.hash_to_bytes(len)
    }
    #[inline]
    fn hash_vec_to_length(input: &[Self], len: usize) -> Vec<Vec<u8>> where Self: Sized {
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<Block>() {
            unsafe {
                let input = std::slice::from_raw_parts(input.as_ptr() as *const Block, input.len());
                if len > 16 {
                    input.iter().map(|each| each.hash_to_bytes(len)).collect()
                } else {
                    let mut out = vec![Block::default(); input.len()];
                    crate::aes::fixed_aes_hash(input, &mut out);
                    out.iter().map(|each| {
                        each.0.to_le_bytes()[..len].to_vec()
                    }).collect()
                }
            }
        } else {
            input.iter().map(|each| each.hash_to_length(len)).collect()
        }
    }
    #[inline]
    fn hash_pair_vec_to_length(input: &[(Self, Self)], len: usize) -> Vec<(Vec<u8>, Vec<u8>)> where Self: Sized {
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<Block>() {
            unsafe {
                if len > 16 {
                    input.iter().map(|each| (each.0.hash_to_bytes(len), each.1.hash_to_bytes(len))).collect()
                } else {
                    let slice = std::slice::from_raw_parts(input.as_ptr() as *const Block, input.len() * 2);
                    let mut out = vec![Block::default(); slice.len()];
                    crate::aes::fixed_aes_hash(slice, &mut out);
                    (0..input.len()).map(|i| {
                        let i = i * 2;
                        (out[i].0.to_le_bytes()[..len].to_vec(), out[i+1].0.to_le_bytes()[..len].to_vec())
                    }).collect()
                }
            }
        } else {
            input.iter().map(|each| (each.0.hash_to_bytes(len), each.1.hash_to_bytes(len))).collect()
        }
    }
}

impl<T> HashToLengthed<BitString> for T where T: Hashable {
    #[inline] fn hash_to_length(&self, len: usize) -> BitString {
        self.hash_to_bitstring(len)
    }
}


/* Deprecated 
const RANDOM_GENERATOR_BUFFER_LENGTH: usize = 2048;

/// Seeded random generator.
pub struct Blake3RandomGenerator {
    stream: blake3::OutputReader,
    buffer: [u8; RANDOM_GENERATOR_BUFFER_LENGTH],
    buffer_index: usize,
}

impl Blake3RandomGenerator {

    pub fn from_entropy() -> Self {
        let mut rng = rand::thread_rng();
        let seed = rand::Rng::gen::<u64>(&mut rng);
        Self::new(seed)
    }

    /// Create a new random generator from the given seed.
    pub fn new(seed: u64) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(&seed.to_le_bytes());
        Self {
            stream: hasher.finalize_xof(),
            buffer: [0u8; RANDOM_GENERATOR_BUFFER_LENGTH],
            buffer_index: RANDOM_GENERATOR_BUFFER_LENGTH,
        }
    }

    /// Generate a random usize.
    pub fn gen_usize(&mut self) -> usize {
        if self.buffer_index == RANDOM_GENERATOR_BUFFER_LENGTH {
            self.buffer_index = 0;
            self.stream.fill(&mut self.buffer);
        }
        let ret = unsafe {
            let ptr = self.buffer.as_ptr().add(self.buffer_index) as *const usize;
            *ptr
        };
        self.buffer_index += std::mem::size_of::<usize>();
        ret
    }

    pub fn gen_bool(&mut self) -> bool {
        self.gen_usize() % 2 == 1
    }

    /// Generate a random u64.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub fn gen_u64(&mut self) -> u64 {
        self.gen_usize() as u64
    }

    /// Generate a random u64.
    #[cfg(target_pointer_width = "32")]
    #[inline]
    pub fn gen_u64(&mut self) -> u64 {
        ((self.gen() as u64) << 32) | (self.gen() as u64)
    }

    #[inline]
    pub fn gen_u128(&mut self) -> u128 {
        ((self.gen_u64() as u128) << 64) | (self.gen_u64() as u128)
    }

    #[inline]
    pub fn gen_block(&mut self) -> crate::Block {
        crate::Block::new(self.gen_u128())
    }
}

*/

const BUFFER_LENGTH_U128: usize = 512;
const BUFFER_LENGTH_U64: usize = BUFFER_LENGTH_U128 * 2;

/// Seeded random generator.
/// 
/// This is implemented with a buffer. Every time it samples
/// a new random number/bool etc, it takes an element from the
/// buffer. When the buffer is drained
/// it is refilled with new randomness. If you only use the RNG
/// for a few times, don't use this struct since filling the
/// buffer is expensive.
pub struct BufferedRandomGenerator {
    counter: u128,
    encryptor: aes::Aes128,
    buffer: Box<[Block; BUFFER_LENGTH_U128]>,
    pointer: usize,
}

impl BufferedRandomGenerator {

    /// Create a new random generator from the given seed.
    pub fn new(seed: Block) -> Self {
        use aes::cipher::KeyInit;
        let key = <[u8; 16]>::from(seed);
        let key = aes::cipher::generic_array::GenericArray::from(key);
        let encryptor = aes::Aes128::new(&key);
        Self { counter: 0, encryptor, buffer: Box::new([Block(0); BUFFER_LENGTH_U128]), pointer: BUFFER_LENGTH_U64 }
    }

    /// Create a new random generator from entropy.
    pub fn from_entropy() -> Self {
        let mut rng = rand::thread_rng();
        let seed = rand::Rng::gen::<u128>(&mut rng);
        Self::new(Block::from(seed))
    }

    /// Refill RNG buffer.
    fn refill(&mut self) {
        use aes::cipher::BlockEncrypt;
        // fill buffer with counter, counter+1, ...
        for each in self.buffer.iter_mut() {
            *each = Block::from(self.counter);
            self.counter = self.counter.wrapping_add(1);
        }
        // encrypt the buffer
        unsafe {
            let buffer_aes = std::slice::from_raw_parts_mut(self.buffer.as_mut_ptr() as *mut aes::Block, BUFFER_LENGTH_U128);
            self.encryptor.encrypt_blocks(buffer_aes);
        }
        // reset pointer
        self.pointer = 0;
    }

    /// Generate a u64
    #[inline]
    pub fn gen_u64(&mut self) -> u64 {
        // cast buffer as *u64 and read the pointer-th u64
        if self.pointer == BUFFER_LENGTH_U64 {
            self.refill();
        }
        let ret = unsafe {
            let ptr = self.buffer.as_ptr() as *const u64;
            *ptr.add(self.pointer)
        };
        self.pointer += 1;
        ret
    }

    /// Generate a usize
    #[inline]
    pub fn gen_usize(&mut self) -> usize {self.gen_u64() as usize}

    /// Generate a f64 between [0, 1)
    #[inline]
    pub fn gen_f64(&mut self) -> f64 {
        let x = self.gen_u64();
        x as f64 / (u64::MAX as f64 + 1.0)
    }

    #[inline]
    /// alias for `gen_usize``
    pub fn get(&mut self) -> usize {self.gen_usize()}

    /// Generate a block
    #[inline]
    pub fn gen_block(&mut self) -> Block {
        while self.pointer & 1 == 1 {
            self.pointer += 1;
        }
        if self.pointer == BUFFER_LENGTH_U64 {
            self.refill();
        }
        let ret = self.buffer[self.pointer >> 1];
        self.pointer += 2;
        ret
    }

    /// Generate a u128
    #[inline]
    pub fn gen_u128(&mut self) -> u128 {self.gen_block().0}

    /// Generate a bool
    #[inline]
    pub fn gen_bool(&mut self) -> bool {self.gen_u64() % 2 == 1}

    /// Produces a reader that reads u8s.
    pub fn as_u8(self) -> RandomGeneratorU8Adapter {
        RandomGeneratorU8Adapter { generator: self, offset: BUFFER_LENGTH_U128 * 16 }
    }

    /// Produces a reader that reads u32s.
    pub fn as_u32(self) -> RandomGeneratorU32Adapter {
        RandomGeneratorU32Adapter { generator: self, offset: BUFFER_LENGTH_U128 * 4 }
    }

}

/// A reader from a random generator that reads u8s.
pub struct RandomGeneratorU8Adapter {
    generator: BufferedRandomGenerator,
    offset: usize,
}

impl RandomGeneratorU8Adapter {
    /// Get the next u8.
    #[inline(always)]
    pub fn next(&mut self) -> u8 {
        if self.offset == BUFFER_LENGTH_U128 * 16 {
            self.generator.refill();
            self.offset = 0;
        }
        let ret = unsafe {
            let ptr = (self.generator.buffer.as_ptr() as *const u8).add(self.offset);
            *ptr
        };
        self.offset += 1;
        ret
    }
}

/// A reader from a random generator that reads u32s.
pub struct RandomGeneratorU32Adapter {
    generator: BufferedRandomGenerator,
    offset: usize,
}

impl RandomGeneratorU32Adapter {
    /// Get the next u32.
    #[inline(always)]
    pub fn next(&mut self) -> u32 {
        if self.offset == BUFFER_LENGTH_U128 * 4 {
            self.generator.refill();
            self.offset = 0;
        }
        let ret = unsafe {
            let ptr = (self.generator.buffer.as_ptr() as *const u32).add(self.offset);
            *ptr
        };
        self.offset += 1;
        ret
    }
}


/// Seeded random generator without buffer
/// 
/// This is more lightweight than `RandomGenerator`,
/// so used to generate shorter randomness.
pub struct RandomGenerator {
    reader: blake3::OutputReader,
}

impl RandomGenerator {
    #[allow(missing_docs)] #[inline]
    pub fn from_raw_parts(reader: blake3::OutputReader) -> Self {
        Self { reader }
    }
    #[allow(missing_docs)] #[inline]
    pub fn from_seed(x: Block) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(&x.0.to_le_bytes());
        Self {
            reader: hasher.finalize_xof(),
        }
    }
    #[allow(missing_docs)] #[inline]
    pub fn from_entropy() -> Self {
        let mut rng = rand::thread_rng();
        let seed = rand::Rng::gen::<u128>(&mut rng);
        Self::from_seed(Block::from(seed))
    }

    #[allow(missing_docs)] #[inline]
    pub fn gen_usize(&mut self) -> usize {
        self.gen_u64() as usize
    }
    #[allow(missing_docs)] #[inline]
    pub fn gen_u128(&mut self) -> u128 {
        let mut ret = [0u8; 16];
        self.reader.fill(&mut ret);
        u128::from_le_bytes(ret)
    }
    #[allow(missing_docs)] #[inline]
    pub fn gen_block(&mut self) -> Block {
        Block::from(self.gen_u128())
    }
    #[allow(missing_docs)] #[inline]
    pub fn gen_u64(&mut self) -> u64 {
        let mut ret = [0u8; 8];
        self.reader.fill(&mut ret);
        u64::from_le_bytes(ret)
    }
    #[allow(missing_docs)] #[inline]
    pub fn gen_u32(&mut self) -> u32 {
        let mut ret = [0u8; 4];
        self.reader.fill(&mut ret);
        u32::from_le_bytes(ret)
    }
    #[allow(missing_docs)] #[inline]
    pub fn gen_u8(&mut self) -> u8 {
        let mut ret = [0u8; 1];
        self.reader.fill(&mut ret);
        ret[0]
    }

    #[allow(missing_docs)] #[inline]
    pub fn gen_block_array<const C: usize>(&mut self) -> [Block; C] {
        let mut ret = [Block::default(); C];
        self.reader.fill(unsafe {
            std::slice::from_raw_parts_mut(ret.as_mut_ptr() as *mut u8, C * std::mem::size_of::<Block>())
        });
        ret
    }
    #[allow(missing_docs)] #[inline]
    pub fn gen_usize_array<const C: usize>(&mut self) -> [usize; C] {
        let mut ret = [0usize; C];
        self.reader.fill(unsafe {
            std::slice::from_raw_parts_mut(ret.as_mut_ptr() as *mut u8, C * std::mem::size_of::<usize>())
        });
        ret
    }
    #[allow(missing_docs)] #[inline]
    pub fn gen_u64_array<const C: usize>(&mut self) -> [u64; C] {
        let mut ret = [0u64; C];
        self.reader.fill(unsafe {
            std::slice::from_raw_parts_mut(ret.as_mut_ptr() as *mut u8, C * std::mem::size_of::<u64>())
        });
        ret
    }
    #[allow(missing_docs)] #[inline]
    pub fn gen_u32_array<const C: usize>(&mut self) -> [u32; C] {
        let mut ret = [0u32; C];
        self.reader.fill(unsafe {
            std::slice::from_raw_parts_mut(ret.as_mut_ptr() as *mut u8, C * std::mem::size_of::<u32>())
        });
        ret
    }
    #[allow(missing_docs)] #[inline]
    pub fn gen_u8_array<const C: usize>(&mut self) -> [u8; C] {
        let mut ret = [0u8; C];
        self.reader.fill(&mut ret);
        ret
    }

    #[allow(missing_docs)] #[inline]
    pub fn gen_block_vector(&mut self, length: usize) -> Vec<Block> {
        let mut ret = vec![Block::default(); length];
        self.reader.fill(unsafe {
            std::slice::from_raw_parts_mut(ret.as_mut_ptr() as *mut u8, length * std::mem::size_of::<Block>())
        });
        ret
    }
    #[allow(missing_docs)] #[inline]
    pub fn gen_usize_vector(&mut self, length: usize) -> Vec<usize> {
        let mut ret = vec![0usize; length];
        self.reader.fill(unsafe {
            std::slice::from_raw_parts_mut(ret.as_mut_ptr() as *mut u8, length * std::mem::size_of::<usize>())
        });
        ret
    }
    #[allow(missing_docs)] #[inline]
    pub fn gen_u64_vector(&mut self, length: usize) -> Vec<u64> {
        let mut ret = vec![0u64; length];
        self.reader.fill(unsafe {
            std::slice::from_raw_parts_mut(ret.as_mut_ptr() as *mut u8, length * std::mem::size_of::<u64>())
        });
        ret
    }
    #[allow(missing_docs)] #[inline]
    pub fn gen_u32_vector(&mut self, length: usize) -> Vec<u32> {
        let mut ret = vec![0u32; length];
        self.reader.fill(unsafe {
            std::slice::from_raw_parts_mut(ret.as_mut_ptr() as *mut u8, length * std::mem::size_of::<u32>())
        });
        ret
    }
    #[allow(missing_docs)] #[inline]
    pub fn gen_u8_vector(&mut self, length: usize) -> Vec<u8> {
        let mut ret = vec![0u8; length];
        self.reader.fill(&mut ret);
        ret
    }

    #[allow(missing_docs)] #[inline]
    pub fn gen_bytes(&mut self, length: usize) -> Vec<u8> {
        self.gen_u8_vector(length)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn random_generator() {
        let mut a = 1u8.to_buffered_random_generator();
        let mut b = 1u8.to_buffered_random_generator();
        for _ in 0..100 {
            assert_eq!(a.gen_usize(), b.gen_usize());
        }
        let mut a = 1u8.to_buffered_random_generator();
        let mut b = 32u8.to_buffered_random_generator();
        for _ in 0..100 {
            assert_ne!(a.gen_usize(), b.gen_usize());
        }
    }

    #[test]
    #[ignore = "only to test performance of blake3 and chacha20"]
    fn compare_blake3_chacha20() {
        use crate::utils::TimerOnce;
        use rand::{SeedableRng, RngCore};
        
        let len = 1048576 * 16;
        println!("Hash to {} bytes", len);

        let timer = TimerOnce::new();
        let mut blake3_hasher = blake3::Hasher::new();
        blake3_hasher.update(&[1u8; 32]);
        let mut buffer = vec![0u8; len];
        blake3_hasher.finalize_xof().fill(&mut buffer);
        timer.finish("blake3");

        let timer = TimerOnce::new();
        let mut chacha20_hasher = rand_chacha::ChaCha20Rng::from_seed([1u8; 32]);
        let mut buffer = vec![0u8; len];
        chacha20_hasher.fill_bytes(&mut buffer);
        timer.finish("chacha20");

        let count = 1048576;
        let len = 16;
        println!("Hash to {} bytes {} times", len, count);

        let timer = TimerOnce::new();
        let blake3_hasher = blake3::Hasher::new();
        let mut blake3_output_reader = blake3_hasher.finalize_xof();
        for _ in 0..count {
            let mut buffer = vec![0u8; len];
            blake3_output_reader.fill(&mut buffer);
        }
        timer.finish("blake3");

        let mut chacha20_hasher = rand_chacha::ChaCha20Rng::from_seed([1u8; 32]);
        let timer = TimerOnce::new();
        for _ in 0..count {
            let mut buffer = vec![0u8; len];
            chacha20_hasher.fill_bytes(&mut buffer);
        }
        timer.finish("chacha20");
    }
    
}

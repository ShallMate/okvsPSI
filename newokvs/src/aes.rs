//! Some util functions using AES as hash

use crate::Block;

mod naive {
    use aes::cipher::BlockEncrypt;
    use aes::cipher::KeyInit;
    use lazy_static::lazy_static;
    use crate::Block;

    lazy_static! {
        static ref AES_HASHER: aes::Aes128 = aes::Aes128::new(&aes::cipher::generic_array::GenericArray::from([4u8; 16]));
        static ref AES0: aes::Aes128 = aes::Aes128::new(&aes::cipher::generic_array::GenericArray::from([1u8; 16]));
        static ref AES1: aes::Aes128 = aes::Aes128::new(&aes::cipher::generic_array::GenericArray::from([2u8; 16]));
    }

    #[inline]
    pub fn hash_block_to_block(block: &Block) -> Block {
        let mut ret = *block;
        unsafe {
            let ptr = (&mut ret) as *mut Block as *mut aes::Block;
            AES_HASHER.encrypt_block(&mut *ptr);
        }
        ret ^= *block;
        ret
    }

    pub fn fixed_aes_encrypt_inplace(x: &mut [Block]) {
        unsafe {
            let blocks = std::slice::from_raw_parts_mut(
                x.as_ptr() as *mut Block as *mut aes::Block,
                x.len(),
            );
            AES_HASHER.encrypt_blocks(blocks);
        }
    }

    #[allow(dead_code)]
    pub fn fixed_aes_encrypt_single_inplace(x: &mut Block) {
        unsafe {
            let block = &mut *(x as *mut Block as *mut aes::Block);
            AES_HASHER.encrypt_block(block);
        }
    }

    pub fn fixed_aes_encrypt(x: &[Block], y: &mut [Block]) {
        unsafe {
            let in_buffer = std::slice::from_raw_parts(
                x.as_ptr() as *const Block as *const aes::Block,
                x.len(),
            );
            let out_buffer = std::slice::from_raw_parts_mut(
                y.as_ptr() as *mut Block as *mut aes::Block,
                y.len(),
            );
            AES_HASHER.encrypt_blocks_b2b(in_buffer, out_buffer).unwrap();
        }
    }

    #[inline]
    pub fn fixed_aes_encrypt_single(x: &Block, y: &mut Block) {
        unsafe {
            let in_block = &*(x as *const Block as *const aes::Block);
            let out_block = &mut *(y as *mut Block as *mut aes::Block);
            AES_HASHER.encrypt_block_b2b(in_block, out_block);
        }
    }

    #[allow(unused)]
    pub fn branch_aes_encrypt_inplace(branch: usize, x: &mut [Block]) {
        unsafe {
            let blocks = std::slice::from_raw_parts_mut(
                x.as_ptr() as *mut Block as *mut aes::Block,
                x.len(),
            );
            if branch == 0 {
                AES0.encrypt_blocks(blocks);
            } else {
                AES1.encrypt_blocks(blocks);
            }
        }
    }
    
    pub fn branch_aes_encrypt(branch: usize, x: &[Block], y: &mut [Block]) {
        unsafe {
            let in_buffer = std::slice::from_raw_parts(
                x.as_ptr() as *const Block as *const aes::Block,
                x.len(),
            );
            let out_buffer = std::slice::from_raw_parts_mut(
                y.as_ptr() as *mut Block as *mut aes::Block,
                y.len(),
            );
            if branch == 0 {
                AES0.encrypt_blocks_b2b(in_buffer, out_buffer).unwrap();
            } else {
                AES1.encrypt_blocks_b2b(in_buffer, out_buffer).unwrap();
            }
        }
    }

}

// only sse2
#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[allow(unused)]
mod opt {
    use core::arch::x86_64::*;
    use crate::Block;
    use lazy_static::lazy_static;
    pub struct AesKey([Block; 11]);

    macro_rules! expand_assist {
        ($v1:expr, $v2:expr, $v3:expr, $v4:expr, $shuff_const:expr, $aes_const:expr) => {
            $v2 = _mm_aeskeygenassist_si128($v4, $aes_const);                   
            $v3 = _mm_castps_si128(_mm_shuffle_ps(_mm_castsi128_ps($v3),        
                                                 _mm_castsi128_ps($v1), 16));  
            $v1 = _mm_xor_si128($v1,$v3);                                        
            $v3 = _mm_castps_si128(_mm_shuffle_ps(_mm_castsi128_ps($v3),        
                                                 _mm_castsi128_ps($v1), 140)); 
            $v1 = _mm_xor_si128($v1,$v3);                                        
            $v2 = _mm_shuffle_epi32($v2,$shuff_const);                           
            $v1 = _mm_xor_si128($v1,$v2)
        };
    }

    #[inline]
    fn set_encrypt_key(userkey: Block) -> AesKey {
        unsafe {
            let mut x0: __m128i = _mm_setzero_si128();
            let mut x1: __m128i = _mm_setzero_si128();
            let mut x2: __m128i = _mm_setzero_si128();
            let mut kp = [_mm_setzero_si128(); 11];
            x0 = _mm_loadu_si128(&userkey as *const Block as *const __m128i);
            kp[0] = x0;
            expand_assist!(x0, x1, x2, x0, 255, 1);
            kp[1] = x0;
            expand_assist!(x0, x1, x2, x0, 255, 2);
            kp[2] = x0;
            expand_assist!(x0, x1, x2, x0, 255, 4);
            kp[3] = x0;
            expand_assist!(x0, x1, x2, x0, 255, 8);
            kp[4] = x0;
            expand_assist!(x0, x1, x2, x0, 255, 16);
            kp[5] = x0;
            expand_assist!(x0, x1, x2, x0, 255, 32);
            kp[6] = x0;
            expand_assist!(x0, x1, x2, x0, 255, 64);
            kp[7] = x0;
            expand_assist!(x0, x1, x2, x0, 255, 128);
            kp[8] = x0;
            expand_assist!(x0, x1, x2, x0, 255, 27);
            kp[9] = x0;
            expand_assist!(x0, x1, x2, x0, 255, 54);
            kp[10] = x0;
            std::mem::transmute(kp)
        }
    }

    lazy_static! {
        pub static ref AES_HASHER: AesKey = set_encrypt_key(Block(0x4444444444444444u128));
        pub static ref AES0: AesKey = set_encrypt_key(Block(0x1111111111111111u128));
        pub static ref AES1: AesKey = set_encrypt_key(Block(0x2222222222222222u128));
    }

    impl AesKey {

        #[inline]
        pub fn encrypt_block(&self, blk: &mut Block) {
            let blk = blk as *mut Block as *mut __m128i;
            let k = self.0.as_ptr() as *const __m128i;
            unsafe {
                *blk = _mm_xor_si128(*blk, *k);
                for i in 1..10 {
                    *blk = _mm_aesenc_si128(*blk, *k.add(i));
                }
                *blk = _mm_aesenclast_si128(*blk, *k.add(10));
            }
        }

        #[inline]
        pub fn encrypt_blocks(&self, blks: &mut [Block]) {
            let count = blks.len();
            let first = blks.as_mut_ptr() as *mut __m128i;
            let k = self.0.as_ptr() as *const __m128i;
            unsafe {
                let mut blks = first;
                for i in 0..count {
                    *blks = _mm_xor_si128(*blks, *k);
                    blks = blks.add(1);
                }
                blks = first;
                for i in 1..10 {
                    for j in 0..count {
                        *blks = _mm_aesenc_si128(*blks, *k.add(i));
                        blks = blks.add(1);
                    }
                    blks = first;
                }
                for j in 0..count {
                    *blks = _mm_aesenclast_si128(*blks, *k.add(10));
                    blks = blks.add(1);
                }
            }
        }

        #[inline]
        pub fn encrypt_blocks_b2b(&self, blks: &[Block], out: &mut [Block]) {
            let count = blks.len();
            assert_eq!(count, out.len());
            let out_first = out.as_mut_ptr() as *mut __m128i;
            let blks_first = blks.as_ptr() as *const __m128i;
            let k = self.0.as_ptr() as *const __m128i;
            unsafe {
                let mut blks = blks_first;
                let mut out = out_first;
                for i in 0..count {
                    *out = _mm_xor_si128(*blks, *k);
                    blks = blks.add(1);
                    out = out.add(1);
                }
                out = out_first;
                for i in 1..10 {
                    for j in 0..count {
                        *out = _mm_aesenc_si128(*out, *k.add(i));
                        out = out.add(1);
                    }
                    out = out_first;
                }
                for j in 0..count {
                    *out = _mm_aesenclast_si128(*out, *k.add(10));
                    out = out.add(1);
                }
            }
        }

    }

    

    #[inline]
    pub fn hash_block_to_block(block: &Block) -> Block {
        use aes::cipher::BlockEncrypt;
        let mut ret = *block;
        AES_HASHER.encrypt_block(&mut ret);
        ret ^= *block;
        ret
    }

    pub fn fixed_aes_encrypt_inplace(x: &mut [Block]) {
        AES_HASHER.encrypt_blocks(x);
    }

    pub fn fixed_aes_encrypt(x: &[Block], y: &mut [Block]) {
        AES_HASHER.encrypt_blocks_b2b(x, y);
    }


    pub fn branch_aes_encrypt_inplace(branch: usize, x: &mut [Block]) {
        if branch == 0 {
            AES0.encrypt_blocks(x);
        } else {
            AES1.encrypt_blocks(x);
        }
    }
    
    pub fn branch_aes_encrypt(branch: usize, x: &[Block], y: &mut [Block]) {
        if branch == 0 {
            AES0.encrypt_blocks_b2b(x, y);
        } else {
            AES1.encrypt_blocks_b2b(x, y);
        }
    }


}



pub use naive::*;

pub fn fixed_aes_hash(x: &[Block], y: &mut [Block]) {
    debug_assert_eq!(x.len(), y.len());
    // enc y
    fixed_aes_encrypt(x, y);
    // xor x to y
    for i in 0..x.len() {
        y[i] ^= x[i];
    }
}

pub fn fixed_aes_hash_single(x: &Block, y: &mut Block) {
    // enc y
    fixed_aes_encrypt_single(x, y);
    // xor x to y
    *y ^= *x;
}

pub fn branch_aes_hash(branch: usize, x: &[Block], y: &mut [Block]) {
    debug_assert_eq!(x.len(), y.len());
    // enc y
    branch_aes_encrypt(branch, x, y);
    // xor x to y
    for i in 0..x.len() {
        y[i] ^= x[i];
    }
}

pub fn fixed_aes_hash_block_to_block_vecs(x: &[Block], len: usize) -> Vec<Block> {
    let mut y = Vec::with_capacity(x.len() * len);
    for (&x, y_chunk) in x.iter().zip(y.chunks_exact_mut(len)) {
        let mut x = x;
        for y in y_chunk.iter_mut() {
            *y = x;
            x.0 = x.0.wrapping_add(1);
        }
    }
    let mut out = vec![Block(0); y.len()];
    fixed_aes_hash(&y, &mut out);
    out
}

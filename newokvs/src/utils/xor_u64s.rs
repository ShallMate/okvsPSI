
pub(self) mod native {
    #[inline]
    pub fn xor_u64s_inplace(x: *mut u64, y: *const u64, len: usize) {
        unsafe {
            for i in 0..len {
                *x.add(i) ^= *y.add(i);
            }
        }
    }
}

/*
#[cfg(target_feature = "avx512f")]
pub(self) mod avx512f {
    #[inline]
    pub unsafe fn xor_u64s_inplace(x: *mut u64, y: *const u64, len: usize) {
        use std::arch::x86_64::*;
        let mut i = 0;
        let remainder = len % 8;
        while i < len - remainder {
            let x_vec = _mm512_loadu_si512(x.add(i) as *const _);
            let y_vec = _mm512_loadu_si512(y.add(i) as *const _);
            let res = _mm512_xor_si512(x_vec, y_vec);
            _mm512_storeu_si512(x.add(i) as *mut _, res);
            i += 8;
        }
        if remainder > 0 {
            super::native::xor_u64s_inplace(x.add(i), y.add(i), remainder);
        }
    }
}
*/

#[cfg(target_feature = "avx2")]
pub(self) mod avx2 {
    #[inline]
    pub unsafe fn xor_u64s_inplace(x: *mut u64, y: *const u64, len: usize) {
        use std::arch::x86_64::*;
        let mut i = 0;
        let remainder = len % 4;
        while i < len - remainder {
            let x_vec = _mm256_loadu_si256(x.add(i) as *const _);
            let y_vec = _mm256_loadu_si256(y.add(i) as *const _);
            let res = _mm256_xor_si256(x_vec, y_vec);
            _mm256_storeu_si256(x.add(i) as *mut _, res);
            i += 4;
        }
        if remainder > 0 {
            super::native::xor_u64s_inplace(x.add(i), y.add(i), remainder);
        }
    }
}

#[cfg(target_feature = "sse2")]
pub(self) mod sse2 {
    #[inline]
    pub unsafe fn xor_u64s_inplace(x: *mut u64, y: *const u64, len: usize) {
        use std::arch::x86_64::*;
        let mut i = 0;
        let remainder = len % 2;
        while i < len - remainder {
            let x_vec = _mm_loadu_si128(x.add(i) as *const _);
            let y_vec = _mm_loadu_si128(y.add(i) as *const _);
            let res = _mm_xor_si128(x_vec, y_vec);
            _mm_storeu_si128(x.add(i) as *mut _, res);
            i += 2;
        }
        if remainder > 0 {
            super::native::xor_u64s_inplace(x.add(i), y.add(i), remainder);
        }
    }
}


// #[cfg(target_feature = "avx512f")]
// pub use avx512f::xor_u64s_inplace;

#[cfg(all(
    target_feature = "avx2", 
    // not(target_feature = "avx512f")
))]
pub use avx2::xor_u64s_inplace;

#[cfg(all(
    target_feature = "sse2", 
    not(target_feature = "avx2"), 
    // not(target_feature = "avx512f")
))]
pub use sse2::xor_u64s_inplace;

#[cfg(not(any(
    // target_feature = "avx512f", 
    target_feature = "avx2", 
    target_feature = "sse2"
)))]
pub use native::xor_u64s_inplace;

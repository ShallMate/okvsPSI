use crate::BitString;


/// Wrapper of u128 aligned to 16 bytes
#[repr(align(16))]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct Block(pub u128);

impl std::fmt::Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write hex split as 4 x u32s
        f.write_fmt(format_args!("{:08x}_{:08x}_{:08x}_{:08x}", 
            (self.0 >> 96) as u32,
            (self.0 >> 64) as u32, 
            (self.0 >> 32) as u32, 
            self.0 as u32, 
        ))
    }
}

impl Block {
    /// Constructor from u128
    #[inline]
    pub fn new(x: u128) -> Self {
        Block(x)
    }
    /// Convert to a bitstring. The least significant bit is the first bit of the bitstring.
    pub fn to_bitstring(&self) -> BitString {
        let mut ret = BitString::new_zeros(128);
        for i in 0..128 {
            ret.set(i, (self.0 >> i) & 1 != 0);
        }
        ret
    }
    #[inline(always)]
    pub fn u0(&self) -> u64 {
        unsafe {*((&self.0) as *const u128 as *const u64)}
    }
    #[inline(always)]
    pub fn u1(&self) -> u64 {
        unsafe {*((&self.0) as *const u128 as *const u64).add(1)}
    }
    #[inline(always)]
    pub fn set_u0(&mut self, x: u64) {
        unsafe {*((&mut self.0) as *mut u128 as *mut u64) = x;}
    }
    #[inline(always)]
    pub fn set_u1(&mut self, x: u64) {
        unsafe {*((&mut self.0) as *mut u128 as *mut u64).add(1) = x;}
    }

    #[inline(always)]
    pub fn i0(&self) -> i64 {
        unsafe {*((&self.0) as *const u128 as *const i64)}
    }
    #[inline(always)]
    pub fn i1(&self) -> i64 {
        unsafe {*((&self.0) as *const u128 as *const i64).add(1)}
    }
    #[inline(always)]
    pub fn set_i0(&mut self, x: i64) {
        unsafe {*((&mut self.0) as *mut u128 as *mut i64) = x;}
    }
    #[inline(always)]
    pub fn set_i1(&mut self, x: i64) {
        unsafe {*((&mut self.0) as *mut u128 as *mut i64).add(1) = x;}
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
    #[inline(always)]
    pub fn as_m128i(&self) -> std::arch::x86_64::__m128i {
        unsafe {std::mem::transmute(*self)}
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
    #[inline(always)]
    pub fn from_m128i(x: std::arch::x86_64::__m128i) -> Self {
        unsafe {std::mem::transmute(x)}
    }

    pub const ALL_ONE_BLOCK: Block = Block(u128::max_value());
    pub const ALL_ZERO_BLOCK: Block = Block(0);
    
}

impl From<u128> for Block {
    #[inline]
    fn from(x: u128) -> Self {
        Block(x)
    }
}

impl From<u64> for Block {
    #[inline]
    fn from(x: u64) -> Self {
        Block(x as u128)
    }
}

impl From<u32> for Block {
    #[inline]
    fn from(x: u32) -> Self {
        Block(x as u128)
    }
}

impl From<bool> for Block {
    #[inline]
    fn from(x: bool) -> Self {
        Block(x as u128)
    }
}

impl From<Block> for u128 {
    #[inline]
    fn from(x: Block) -> Self {
        x.0
    }
}

impl From<Block> for [u8; 16] {
    fn from(x: Block) -> Self {
        x.0.to_le_bytes()
    }
}

impl From<[u8; 16]> for Block {
    fn from(x: [u8; 16]) -> Self {
        Block(u128::from_le_bytes(x))
    }
}

impl From<Block> for [u16; 8] {
    fn from(x: Block) -> Self {
        let bytes = x.0.to_le_bytes();
        let mut result = [0; 8];
        for i in 0..8 {
            result[i] = u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]);
        }
        result
    }
}

impl From<[u16; 8]> for Block {
    fn from(x: [u16; 8]) -> Self {
        let mut bytes = [0; 16];
        for i in 0..8 {
            let bytes_i = x[i].to_le_bytes();
            bytes[2 * i] = bytes_i[0];
            bytes[2 * i + 1] = bytes_i[1];
        }
        Block(u128::from_le_bytes(bytes))
    }
}

impl std::ops::Add for Block {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Block(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Block {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Block(self.0 - rhs.0)
    }
}

impl std::ops::Mul for Block {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Block(self.0 * rhs.0)
    }
}

impl std::ops::Div for Block {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self {
        Block(self.0 / rhs.0)
    }
}

impl std::ops::Rem for Block {
    type Output = Self;
    #[inline]
    fn rem(self, rhs: Self) -> Self {
        Block(self.0 % rhs.0)
    }
}

impl std::ops::AddAssign for Block {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::Not for Block {
    type Output = Self;
    #[inline]
    fn not(self) -> Self {
        Block(!self.0)
    }
}

impl std::ops::BitXor for Block {
    type Output = Self;
    #[inline]
    fn bitxor(self, rhs: Self) -> Self {
        Block(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for Block {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl std::ops::BitAnd for Block {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Block(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for Block {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl std::ops::BitOr for Block {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Block(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for Block {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl std::ops::Shl<usize> for Block {
    type Output = Self;
    #[inline]
    fn shl(self, rhs: usize) -> Self {
        Block(self.0 << rhs)
    }
}

impl std::ops::ShlAssign<usize> for Block {
    #[inline]
    fn shl_assign(&mut self, rhs: usize) {
        *self = *self << rhs;
    }
}

impl std::ops::Shr<usize> for Block {
    type Output = Self;
    #[inline]
    fn shr(self, rhs: usize) -> Self {
        Block(self.0 >> rhs)
    }
}

impl std::ops::ShrAssign<usize> for Block {
    #[inline]
    fn shr_assign(&mut self, rhs: usize) {
        *self = *self >> rhs;
    }
}

impl std::ops::Shl<i32> for Block {
    type Output = Self;
    #[inline]
    fn shl(self, rhs: i32) -> Self {
        Block(self.0 << rhs)
    }
}

impl std::ops::ShlAssign<i32> for Block {
    #[inline]
    fn shl_assign(&mut self, rhs: i32) {
        *self = *self << rhs;
    }
}

impl std::ops::Shr<i32> for Block {
    type Output = Self;
    #[inline]
    fn shr(self, rhs: i32) -> Self {
        Block(self.0 >> rhs)
    }
}

impl std::ops::ShrAssign<i32> for Block {
    #[inline]
    fn shr_assign(&mut self, rhs: i32) {
        *self = *self >> rhs;
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("*{:08x}", self.0 as u32))
    }
}

impl num_traits::SaturatingAdd for Block {
    #[inline]
    fn saturating_add(&self, rhs: &Self) -> Self {
        Block(self.0.saturating_add(rhs.0))
    }
}

impl num_traits::SaturatingSub for Block {
    #[inline]
    fn saturating_sub(&self, rhs: &Self) -> Self {
        Block(self.0.saturating_sub(rhs.0))
    }
}

impl num_traits::SaturatingMul for Block {
    #[inline]
    fn saturating_mul(&self, rhs: &Self) -> Self {
        Block(self.0.saturating_mul(rhs.0))
    }
}

impl num_traits::Saturating for Block {
    #[inline]
    fn saturating_add(self, rhs: Self) -> Self {
        Block(self.0.saturating_add(rhs.0))
    }
    #[inline]
    fn saturating_sub(self, rhs: Self) -> Self {
        Block(self.0.saturating_sub(rhs.0))
    }
}

impl num_traits::WrappingNeg for Block {
    #[inline]
    fn wrapping_neg(&self) -> Self {
        Block(self.0.wrapping_neg())
    }
}

impl num_traits::CheckedAdd for Block {
    #[inline]
    fn checked_add(&self, rhs: &Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Block)
    }
}

impl num_traits::CheckedSub for Block {
    #[inline]
    fn checked_sub(&self, rhs: &Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Block)
    }
}

impl num_traits::CheckedDiv for Block {
    #[inline]
    fn checked_div(&self, rhs: &Self) -> Option<Self> {
        self.0.checked_div(rhs.0).map(Block)
    }
}

impl num_traits::CheckedMul for Block {
    #[inline]
    fn checked_mul(&self, rhs: &Self) -> Option<Self> {
        self.0.checked_mul(rhs.0).map(Block)
    }
}

impl std::cmp::PartialOrd for Block {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl std::cmp::Ord for Block {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl num_traits::Bounded for Block {
    #[inline]
    fn min_value() -> Self {
        Block(u128::min_value())
    }
    #[inline]
    fn max_value() -> Self {
        Block(u128::max_value())
    }
}

impl num_traits::Zero for Block {
    #[inline]
    fn zero() -> Self {
        Block(0)
    }
    #[inline]
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl num_traits::One for Block {
    #[inline]
    fn one() -> Self {
        Block(1)
    }
}

impl num_traits::Num for Block {
    type FromStrRadixErr = <u128 as num_traits::Num>::FromStrRadixErr;
    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        u128::from_str_radix(str, radix).map(Block)
    }
}

impl num_traits::ToPrimitive for Block {
    #[inline]
    fn to_i64(&self) -> Option<i64> {
        self.0.to_i64().into()
    }
    #[inline]
    fn to_u64(&self) -> Option<u64> {
        self.0.to_u64().into()
    }
}

impl num_traits::NumCast for Block {
    #[inline]
    fn from<T: num_traits::ToPrimitive>(n: T) -> Option<Self> {
        <u128 as num_traits::NumCast>::from(n).map(Block)
    }
}

impl num_traits::PrimInt for Block {
    #[inline]
    fn count_ones(self) -> u32 {
        self.0.count_ones()
    }
    #[inline]
    fn count_zeros(self) -> u32 {
        self.0.count_zeros()
    }
    #[inline]
    fn leading_zeros(self) -> u32 {
        self.0.leading_zeros()
    }
    #[inline]
    fn trailing_zeros(self) -> u32 {
        self.0.trailing_zeros()
    }
    #[inline]
    fn rotate_left(self, n: u32) -> Self {
        self.0.rotate_left(n).into()
    }
    #[inline]
    fn rotate_right(self, n: u32) -> Self {
        self.0.rotate_right(n).into()
    }
    #[inline]
    fn signed_shl(self, n: u32) -> Self {
        self.0.signed_shl(n).into()
    }
    #[inline]
    fn signed_shr(self, n: u32) -> Self {
        self.0.signed_shr(n).into()
    }
    #[inline]
    fn unsigned_shl(self, n: u32) -> Self {
        self.0.unsigned_shl(n).into()
    }
    #[inline]
    fn unsigned_shr(self, n: u32) -> Self {
        self.0.unsigned_shr(n).into()
    }
    #[inline]
    fn swap_bytes(self) -> Self {
        self.0.swap_bytes().into()
    }
    #[inline]
    fn from_be(x: Self) -> Self {
        u128::from_be(x.0).into()
    }
    #[inline]
    fn from_le(x: Self) -> Self {
        u128::from_le(x.0).into()
    }
    #[inline]
    fn to_be(self) -> Self {
        self.0.to_be().into()
    }
    #[inline]
    fn to_le(self) -> Self {
        self.0.to_le().into()
    }
    #[inline]
    fn pow(self, exp: u32) -> Self {
        self.0.pow(exp).into()
    }
}

impl num_traits::Unsigned for Block {}

pub(self) mod native {
    #[inline(always)]
    pub fn dot_u64_generic<Block>(a: u64, b: &[Block]) -> Block
    where Block: Default + Clone + From<u64> + std::ops::Mul<Output=Block> + std::ops::BitXorAssign {
        let mut out = Block::default();
        if b.len() >= 64 {
            out ^= b[00].clone() * Block::from((a >> 00) & 1);
            out ^= b[01].clone() * Block::from((a >> 01) & 1);
            out ^= b[02].clone() * Block::from((a >> 02) & 1);
            out ^= b[03].clone() * Block::from((a >> 03) & 1);
            out ^= b[04].clone() * Block::from((a >> 04) & 1);
            out ^= b[05].clone() * Block::from((a >> 05) & 1);
            out ^= b[06].clone() * Block::from((a >> 06) & 1);
            out ^= b[07].clone() * Block::from((a >> 07) & 1);
            out ^= b[08].clone() * Block::from((a >> 08) & 1);
            out ^= b[09].clone() * Block::from((a >> 09) & 1);
            out ^= b[10].clone() * Block::from((a >> 10) & 1);
            out ^= b[11].clone() * Block::from((a >> 11) & 1);
            out ^= b[12].clone() * Block::from((a >> 12) & 1);
            out ^= b[13].clone() * Block::from((a >> 13) & 1);
            out ^= b[14].clone() * Block::from((a >> 14) & 1);
            out ^= b[15].clone() * Block::from((a >> 15) & 1);
            out ^= b[16].clone() * Block::from((a >> 16) & 1);
            out ^= b[17].clone() * Block::from((a >> 17) & 1);
            out ^= b[18].clone() * Block::from((a >> 18) & 1);
            out ^= b[19].clone() * Block::from((a >> 19) & 1);
            out ^= b[20].clone() * Block::from((a >> 20) & 1);
            out ^= b[21].clone() * Block::from((a >> 21) & 1);
            out ^= b[22].clone() * Block::from((a >> 22) & 1);
            out ^= b[23].clone() * Block::from((a >> 23) & 1);
            out ^= b[24].clone() * Block::from((a >> 24) & 1);
            out ^= b[25].clone() * Block::from((a >> 25) & 1);
            out ^= b[26].clone() * Block::from((a >> 26) & 1);
            out ^= b[27].clone() * Block::from((a >> 27) & 1);
            out ^= b[28].clone() * Block::from((a >> 28) & 1);
            out ^= b[29].clone() * Block::from((a >> 29) & 1);
            out ^= b[30].clone() * Block::from((a >> 30) & 1);
            out ^= b[31].clone() * Block::from((a >> 31) & 1);
            out ^= b[32].clone() * Block::from((a >> 32) & 1);
            out ^= b[33].clone() * Block::from((a >> 33) & 1);
            out ^= b[34].clone() * Block::from((a >> 34) & 1);
            out ^= b[35].clone() * Block::from((a >> 35) & 1);
            out ^= b[36].clone() * Block::from((a >> 36) & 1);
            out ^= b[37].clone() * Block::from((a >> 37) & 1);
            out ^= b[38].clone() * Block::from((a >> 38) & 1);
            out ^= b[39].clone() * Block::from((a >> 39) & 1);
            out ^= b[40].clone() * Block::from((a >> 40) & 1);
            out ^= b[41].clone() * Block::from((a >> 41) & 1);
            out ^= b[42].clone() * Block::from((a >> 42) & 1);
            out ^= b[43].clone() * Block::from((a >> 43) & 1);
            out ^= b[44].clone() * Block::from((a >> 44) & 1);
            out ^= b[45].clone() * Block::from((a >> 45) & 1);
            out ^= b[46].clone() * Block::from((a >> 46) & 1);
            out ^= b[47].clone() * Block::from((a >> 47) & 1);
            out ^= b[48].clone() * Block::from((a >> 48) & 1);
            out ^= b[49].clone() * Block::from((a >> 49) & 1);
            out ^= b[50].clone() * Block::from((a >> 50) & 1);
            out ^= b[51].clone() * Block::from((a >> 51) & 1);
            out ^= b[52].clone() * Block::from((a >> 52) & 1);
            out ^= b[53].clone() * Block::from((a >> 53) & 1);
            out ^= b[54].clone() * Block::from((a >> 54) & 1);
            out ^= b[55].clone() * Block::from((a >> 55) & 1);
            out ^= b[56].clone() * Block::from((a >> 56) & 1);
            out ^= b[57].clone() * Block::from((a >> 57) & 1);
            out ^= b[58].clone() * Block::from((a >> 58) & 1);
            out ^= b[59].clone() * Block::from((a >> 59) & 1);
            out ^= b[60].clone() * Block::from((a >> 60) & 1);
            out ^= b[61].clone() * Block::from((a >> 61) & 1);
            out ^= b[62].clone() * Block::from((a >> 62) & 1);
            out ^= b[63].clone() * Block::from((a >> 63) & 1);
        } else {
            for i in 0..b.len() {
                out ^= b[i].clone() * Block::from((a >> i) & 1);
            }
        }
        out
    }
}

pub use native::dot_u64_generic;

pub mod okvs;
pub mod newokvs;
pub mod utils;
pub mod hash;
pub mod block;
pub mod aes;
pub mod bitstring;

use bitstring::BitString;
use block::Block;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

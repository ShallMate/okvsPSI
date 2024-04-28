use crate::okvs::OkvsDecoder;
use crate::okvs::OkvsEncoder;
use crate::hash::Hashable;
use crate::Block;

type Bucket = u64;
const SNAP_LEN: usize = 64;

const DEBUG: bool = true;

use crate::utils::xor_u64s_inplace;
use crate::utils::dot_u64_generic;

#[derive(Clone, Debug)]
pub struct OKVS {
    epsilon: f64,
    width: usize,
}

#[inline]
fn hash_row_k<T>(key: &T, count: usize) -> (usize, Vec<Bucket>) where T: Hashable + std::any::Any {
    let mut hash = key.hash_to_hasher().finalize_xof();
    if std::any::TypeId::of::<T>() == std::any::TypeId::of::<Block>() {
        let key = unsafe {*(key as *const T as *const Block)};
        let required_bytes = 8 + count * std::mem::size_of::<Bucket>();
        let required_blocks = (required_bytes + 15) / 16;
        let mut buf = vec![Block::default(); required_blocks];
        for i in 0..required_blocks {
            buf[i] = Block(key.0.wrapping_add(i as u128)).hash_to_block();
        }
        unsafe {
            // take the start 8 bytes of buf
            let buf0 = std::slice::from_raw_parts(
                buf.as_ptr() as *const u8,
                std::mem::size_of::<usize>()
            );
            // take the latter count * 8 bytes of buf
            let buf1 = std::slice::from_raw_parts(
                (buf.as_ptr() as *const u8).add(8),
                count * std::mem::size_of::<Bucket>()
            );
            let mut start_index = 0;
            std::slice::from_raw_parts_mut(
                &mut start_index as *mut usize as *mut u8,
                std::mem::size_of::<usize>()
            ).copy_from_slice(buf0);
            let mut offsets = vec![0 as Bucket; count];
            std::slice::from_raw_parts_mut(
                offsets.as_mut_ptr() as *mut u8,
                count * std::mem::size_of::<Bucket>()
            ).copy_from_slice(buf1);
            (start_index, offsets)
        }
    } else {
        let mut start_index: usize = 0;
        unsafe {
            hash.fill(std::slice::from_raw_parts_mut(
                &mut start_index as *mut usize as *mut u8,
                std::mem::size_of::<usize>()
            ));
        }
        start_index %= count * SNAP_LEN;
        let mut offsets = vec![0 as Bucket; count];
        unsafe {
            hash.fill(std::slice::from_raw_parts_mut(
                offsets.as_mut_ptr() as *mut u8,
                count * std::mem::size_of::<Bucket>()
            ));
        }
        (start_index, offsets)
    }
}

fn row_k<Key>(key: &Key, m: usize, width: usize) -> (usize, Vec<Bucket>) where Key: Hashable + std::any::Any {
    let count = (width - 2 + SNAP_LEN) / SNAP_LEN + 1;
    let (mut start_index, mut offsets) = hash_row_k(key, count);
    start_index %= m - width;
    offsets[0] &= !((1 << (start_index % SNAP_LEN)) - 1);
    let last_index = ((start_index % SNAP_LEN) + width) / SNAP_LEN;
    assert!(last_index >= count - 2);
    if last_index < count {
        offsets[last_index] &= (1 << ((start_index + width) % SNAP_LEN)) - 1;
    }
    if last_index == count - 2 {
        offsets[last_index + 1] = 0;
    }
    (start_index, offsets)
}

impl OKVS {

    pub fn new(epsilon: f64, width: usize) -> Self {
        Self { epsilon, width }
    }
    
    #[allow(unused)]
    fn encode_length(&self, count: usize) -> usize {
        let m = (count as f64 * (1.0 + self.epsilon)).ceil() as usize;
        m
    }

}

impl<Key, Value> OkvsEncoder<Key, Value> for OKVS where
    Key: Hashable + std::any::Any,
    Value: Default + Clone + From<Bucket> + std::ops::Mul<Output=Value> + std::ops::BitXorAssign
{

    fn encode(&self, map: &Vec<(Key, Value)>) -> Vec<Value> {
        use crate::utils::TimerOnce;

        // sanity
        let n = map.len();
        let m = (n as f64 * (1.0 + self.epsilon)).ceil() as usize;
        assert!(m > self.width);

        let mut rows = Vec::with_capacity(n);
        for (key, value) in map {
            let (start_index, offsets) = row_k(key, m, self.width);
            rows.push((start_index, offsets, value.clone()));
        }
        // Sort with first index
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        let mut offsets = Vec::with_capacity(n);
        let mut v = Vec::with_capacity(n);
        let mut start_indices = Vec::with_capacity(n);
        for (start_index, offset, value) in rows {
            start_indices.push(start_index);
            v.push(value);
            offsets.push(offset);
        }
        let timer = TimerOnce::new().tabs(2);
        for i in 0..n {
            // println!("i={:02}", i);
            let i_id = start_indices[i] / SNAP_LEN;
            let mut j = 0;
            let mut found = false;
            for each in &offsets[i] {
                if *each != 0 {
                    found = true;
                    j += each.trailing_zeros() as usize;
                    break;
                }
                j += SNAP_LEN;
            }
            if !found {
                panic!("Matrix is singular");
            }
            for k in (i + 1)..n {
                if start_indices[k] > i_id * SNAP_LEN + j {
                    break;
                }
                let k_id = start_indices[k] / SNAP_LEN;
                let id_offset = k_id - i_id;
                if (offsets[k][j / SNAP_LEN - id_offset] >> (j % SNAP_LEN)) & 1 != 0 {
                    let vi = v[i].clone();
                    v[k] ^= vi;
                    unsafe {xor_u64s_inplace(
                        offsets[k].as_mut_ptr(), 
                        offsets[i].as_ptr().add(id_offset), 
                        offsets[k].len() - id_offset
                    );}
                }
            }
        }
        if DEBUG {timer.finish("Encode time");}
        let mut s = vec![Value::default(); m];
        for i in (0..n).rev() {
            let mut j = 0;
            for each in &offsets[i] {
                if *each != 0 {
                    j += each.trailing_zeros() as usize;
                    break;
                }
                j += SNAP_LEN;
            }
            let mut sum = v[i].clone();
            let i_id = start_indices[i] / SNAP_LEN;
            for k in 0..offsets[i].len() {
                if (i_id + k) * SNAP_LEN >= s.len() {
                    continue;
                }
                let range = &s[(i_id + k) * SNAP_LEN..];
                sum ^= dot_u64_generic(offsets[i][k], range);
            }
            s[i_id * SNAP_LEN + j] = sum;
        }
        s
    }
}

impl<Key, Value> OkvsDecoder<Key, Value> for OKVS where
    Key: Hashable + std::any::Any,
    Value: Default + Clone + From<Bucket> + std::ops::Mul<Output=Value> + std::ops::BitXorAssign
{
    fn decode(&self, okvs: &[Value], key: &Key) -> Value {
        let (start_index, offsets) = row_k(key, okvs.len(), self.width);
        let mut sum = Value::default();
        let i_id = start_index / SNAP_LEN;
        for k in 0..offsets.len() {
            if (i_id + k) * SNAP_LEN >= okvs.len() {
                continue;
            }
            let range = &okvs[(i_id + k) * SNAP_LEN..];
            sum ^= dot_u64_generic(offsets[k], range);
        }
        sum
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::Block;

    #[test]
    pub fn OKVS_encode() {
        let mut map = Vec::new();
        let n: usize = 1024;
        let width: usize = 87;
        let keys = (0..n).collect::<Vec<_>>();
        for &i in &keys {
            map.push((i, Block((i*i) as u128)));
        }
        let encoder = OKVS::new(0.01, width);
        let s = encoder.encode(&map);
        for (key, value) in map {
            assert_eq!(encoder.decode(&s, &key), value, "key = {}", key);
        }
    }

}
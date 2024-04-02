// use std::collections::HashMap;

/// Oblivious key-value store encoder
pub trait OkvsEncoder<Key, Value> {
    // TODO: HashMap instead of Vec
    fn encode(&self, map: &Vec<(Key, Value)>) -> Vec<Value>;
}

/// Oblivious key-value store decoder
pub trait OkvsDecoder<Key, Value> {
    fn decode(&self, okvs: &[Value], key: &Key) -> Value;
    fn decode_many(&self, okvs: &[Value], keys: &[Key]) -> Vec<Value> {
        keys.iter().map(|key| self.decode(okvs, key)).collect()
    }
}
use crate::tx::Tx;
use crate::utilities::Wrap;
use std::collections::*;
use std::hash::Hash;
use typing::constraint::*;

#[derive(Debug, Clone)]
pub struct Aux<I, K, V> {
    pub rdset: HashMap<K, (Option<V>, I)>,
    pub wrset: HashMap<K, (Option<V>, bool)>,
    pub wrpub: bool,
}

impl<I, K: Hash + Eq, V: Clone> Aux<I, K, V> {
    pub fn new() -> Self {
        Self {
            rdset: HashMap::new(),
            wrset: HashMap::new(),
            wrpub: false,
        }
    }
    pub fn read_local(&self, key: &K) -> Option<Option<V>> {
        if self.rdset.contains_key(key) {
            return Some(self.rdset[key].0.clone());
        }
        if self.wrset.contains_key(key) {
            return Some(self.wrset[key].0.clone());
        }
        return None;
    }
}

pub type KVSparkleTx<V, T> = Wrap<T, Aux<<T as Tx<V>>::I, <V as Id>::I, V>>;

impl<V: Clone + Id, T: Tx<V>> KVSparkleTx<V, T>
where
    V::I: Hash + Eq,
{
    pub fn new(tx: T) -> KVSparkleTx<V, T> {
        Wrap { tx, ax: Aux::new() }
    }
}

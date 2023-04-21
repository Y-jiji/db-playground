use dashmap::DashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use typing::tx::*;

pub struct TPool<T: Sized + Tx<V>, V>
where
    T::I: Ord + Debug + Eq + Hash + Copy,
{
    todo: DashMap<T::I, T>,
    done: DashMap<T::I, T>,
    phan: PhantomData<V>,
}

impl<T: Sized + Tx<V>, V> TPool<T, V>
where
    T::I: Ord + Debug + Eq + Hash + Copy,
{
    pub fn new() -> Self {
        Self {
            todo: DashMap::new(),
            done: DashMap::new(),
            phan: PhantomData::<V>,
        }
    }
    pub fn put_todo(&self, txn: T) {
        self.todo.insert(txn.id(), txn);
    }
    pub fn put_done(&self, txn: T) {
        self.done.insert(txn.id(), txn);
    }
    pub fn get_prog(&self, tid: T::I) -> Option<T> {
        use rand::random;
        match self.done.remove(&tid) {
            Some((_, txn)) => return Some(txn),
            None => {}
        };
        match self.todo.remove(&tid) {
            Some((_, txn)) => Some(txn),
            None if random::<bool>() => None,
            None => {
                let key = *(self.todo.iter().next()?.key());
                Some(self.todo.remove(&key)?.1)
            }
        }
    }
}

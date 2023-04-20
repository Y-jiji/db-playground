use parking_lot::Mutex;
use typing::constraint::*;
use typing::rw::*;
use typing::tx::*;
use dashmap::DashMap;
use std::hash::Hash;
use std::collections::HashMap;

pub struct Serial<T, V>
where
    T: Tx<V>,
    V: Id,
    V::I: Hash + Eq,
    T::I: Nat + Hash + Eq,
{
    prog: Mutex<T::I>,
    pool: DashMap<T::I, T>,
    waiting: DashMap<T::I, HashMap<V::I, Option<V>>>,
}

impl<T, V> Serial<T, V>
where
    T: Tx<V>,
    V: Id,
    V::I: Hash + Eq,
    T::I: Nat + Hash + Eq,
{
    pub fn new() -> Self {
        Self {
            prog: Mutex::new(T::I::zero()),
            pool: DashMap::new(),
            waiting: DashMap::new(),
        }
    }
    fn put(&self, txn: T) {
        self.pool.insert(txn.id(), txn);
    }
    fn get(&self) -> Option<T> {
        let prog = self.prog.lock().succ();
        self.pool.remove(&prog).map(|(_, txn)| txn)
    }
    fn proceed(&self) {
        let mut prog = self.prog.lock();
        *prog = prog.succ();
    }
    fn next_tid(&self) -> T::I {
        self.prog.lock().succ()
    }
}

impl<T, V, D> RWControl<V, T, D> for Serial<T, V> 
where
    T: Tx<V>,
    V: Id,
    V::I: Hash + Eq,
    T::I: Nat + Hash + Eq,
    D: RWDurable<V, T>,
    T::Map: Mapper<V::I, V>,
{
    type Err = ();
    fn open(&self, txn: &mut T, dur: &D) -> Result<(), Self::Err> {
        dur.open(txn).map_err(|_|())?;
        self.waiting.insert(txn.id(), HashMap::from([]));
        Ok(())
    }
    fn done(&self, txn: T, end: End, dur: &D) 
    -> Result<(Option<T>, Option<Option<T::Out>>), Self::Err> {
        if txn.id() != self.next_tid() {
            self.put(txn);
            Ok((self.get(), None))
        } else {
            let map = self.waiting.remove(&txn.id()).unwrap().1.into_iter();
            if matches!(end, End::Ready) {
                dur.wr(&txn, Mapper::from_mapping(map)).map_err(|_| ())?;
            }
            dur.done(&txn, end).map_err(|_| ())?;
            self.proceed();
            Ok((self.get(), Some(txn.cl())))
        }
    }
    fn rd(&self, txn: T, prp: T::Prp, dur: &D) 
    -> Result<Option<T>, Self::Err> {
        if txn.id() != self.next_tid() {
            self.put(txn);
            Ok(self.get())
        } else {
            let map = dur.rd(prp).map_err(|_| ())?;
            Ok(Some(txn.rd(map)))
        }
    }
    fn wr(&self, txn: T, map: T::Map, _dur: &D) 
    -> Result<Option<T>, Self::Err> {
        if txn.id() != self.next_tid() {
            self.put(txn);
            Ok(self.get())
        } else {
            for (k, v) in map.into_mapping() {
                self.waiting.get_mut(&txn.id()).unwrap().insert(k, v);
            }
            Ok(Some(txn.wr()))
        }
    }
}
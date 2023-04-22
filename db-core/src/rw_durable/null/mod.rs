use std::hash::Hash;
use typing::constraint::*;
use typing::rw::*;
use typing::tx::*;
use std::marker::PhantomData;

pub struct Null<V: Id, T: Tx<V>>
where
    V: Sync + Clone,
    V::I: Eq + Hash + Sync + Clone,
    T::I: Eq + Hash + Sync + Clone,
    T::Prp: Filter<V>,
    T::Map: Mapper<V::I, V>,
{
    table: dashmap::DashMap<V::I, V>,
    phant: PhantomData<T>,
    null_write: bool,
    latency_rd: u64,
    latency_wr: u64,
}

impl<V: Id, T: Tx<V>> Null<V, T>
where
    V: Sync + Clone,
    V::I: Eq + Hash + Sync + Clone,
    T::I: Eq + Hash + Sync + Clone,
    T::Prp: Filter<V>,
    T::Map: Mapper<V::I, V>,
{
    pub fn new(latency_rd: u64, latency_wr: u64, fake_write: bool) -> Self {
        Self {
            table: dashmap::DashMap::new(),
            null_write: fake_write, latency_rd, latency_wr,
            phant: PhantomData,
        }
    }
}

impl<V: Id, T: Tx<V>> RWDurable<V, T> for Null<V, T>
where
    V: Sync + Clone,
    V::I: Eq + Hash + Sync + Clone,
    T::I: Eq + Hash + Sync + Clone,
    T::Prp: Filter<V> + MaybeIndexer<V::I>,
    T::Map: Mapper<V::I, V>,
{
    type Err = ();
    fn done(&self, _txn: &T, _end: End) -> Result<(), Self::Err> {
        Ok(())
    }
    fn open(&self, _txn: &T) -> Result<(), Self::Err> {
        Ok(())
    }
    fn rd(&self, prp: T::Prp) -> Result<T::Map, Self::Err> {
        if self.null_write { 
            return Ok(Mapper::from_mapping([].into_iter())) ;
        }
        let mut map = vec![];
        for _ in 0..self.latency_rd { std::thread::yield_now() }
        if let Some(prp_iter) = prp.tryc_indexer() {
            for i in prp_iter {
                self.table.view(&i, |i, v| {
                    map.push((i.clone(), Some(v.clone())))
                });
            }
        } else {
            let prp = prp.into_filter();
            self.table.alter_all(|i, v| {
                if (prp)(&v) {
                    map.push((i.clone(), Some(v.clone())));
                }
                return v;
            });
        }
        Ok(Mapper::from_mapping(map.into_iter()))
    }
    fn wr(&self, _txn: &T, map: T::Map) -> Result<(), Self::Err> {
        if self.null_write { 
            return Ok(());
        }
        for _ in 0..self.latency_wr { std::thread::yield_now() }
        for (i, v) in map.into_mapping() {
            match v {
                Some(v) => {self.table.insert(i, v);}, 
                None => {self.table.remove(&i);},
            }
        }
        Ok(())
    }
}

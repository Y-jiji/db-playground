use std::cmp::*;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::Hash;

struct Entry<N, V> {
    // lock + wait list
    lock: Option<N>,
    // version and value
    vals: BTreeMap<N, (Option<V>, BTreeSet<N>)>,
}

impl<N: Ord + Copy + Nat + Hash, V: Clone> Entry<N, V> {
    fn is_empty(&self) -> bool {
        self.lock.is_none() && 
        self.vals.is_empty()
    }
}

// a key-value table with versions and dependencies
pub struct KVTable<N, K, V>
where
    K: Eq + Hash + Sync + Clone,
    N: Sync + Copy + Ord + Eq + Hash + Nat,
    V: Sync + Clone,
{
    inner: dashmap::DashMap<K, Entry<N, V>>,
}
pub enum KVTableErr {
    WouldBlock,
    IsPreempted,
    DepDurable,
}

use crate::constraint::Nat;
use std::fmt::Debug;
use KVTableErr::*;

impl<N, K, V> KVTable<N, K, V>
where
    K: Eq + Hash + Sync + Clone + Debug,
    N: Sync + Copy + Ord + Eq + Hash + Nat + Debug,
    V: Sync + Clone,
{
    /// a new, empty kv table
    pub fn new() -> Self {
        KVTable {
            inner: dashmap::DashMap::new(),
        }
    }
    /// delete a read record on a given version
    pub fn unread(&self, key: &K, rid: &N, wid: &N) {
        let upd = |_key: &K, mut entry: Entry<N, V>| -> Entry<N, V> {
            let val = &mut entry.vals.get_mut(wid);
            let dep = match val {
                None => return entry,
                Some((_, dep)) => dep,
            };
            dep.remove(rid);
            return entry;
        };
        self.inner.alter(key, upd);
    }
    /// delete a written version, wid: writer id
    /// return all reader ids of this entry
    pub fn unwrite(&self, key: &K, wid: &N) -> BTreeSet<N> {
        let mut out = BTreeSet::new();
        let upd = |_key: &K, mut entry: Entry<N, V>| -> Entry<N, V> {
            let dep = match entry.vals.remove(wid) {
                Some((_val, dep)) => dep,
                None => return entry,
            };
            out = dep;
            return entry;
        };
        self.inner.alter(key, upd);
        return out;
    }
    /// remove a write lock if there is some
    pub fn unwlock(&self, key: &K, wid: &N) {
        let upd = |_key: &K, mut entry: Entry<N, V>| -> Entry<N, V> {
            if entry.lock == Some(*wid) {
                entry.lock = None
            }
            return entry;
        };
        self.inner.alter(key, upd);
    }
    /// scan every entry, return a list of values
    // pub fn scan<'a>(&self, _prp: Box<dyn Fn(&V) -> bool + 'a>, _rid: N) -> Vec<(K, (Option<V>, N))> {
    //     todo!()
    // }
    /// read an entry, return a value
    pub fn read(&self, key: K, rid: N)
    -> Result<(Option<V>, N), KVTableErr> {
        let mut out = Err(DepDurable);
        let upd = |entry: &mut Entry<N, V>| {
            out = match entry.vals.range_mut(..rid).last() {
                Some((wid, (val, deps))) => {
                    deps.insert(rid);
                    Ok((val.clone(), *wid))
                },
                None => {
                    entry.vals.entry(N::zero())
                        .and_modify(|(_, deps)| { deps.insert(rid); })
                        .or_insert((None, BTreeSet::from([rid])));
                    Err(DepDurable)
                },
            };
        };
        self.inner.entry(key)
            .and_modify(upd)
            .or_insert(Entry {
                lock: None,
                vals: [(N::zero(), (None, [rid].into()))].into(),
            });
        return out;
    }
    /// write a value to this entry
    pub fn write(&self, key: &K, val: Option<V>, wid: N) 
    -> Result<BTreeSet<N>, KVTableErr> {
        let mut out = Ok(BTreeSet::new());
        let upd = |_key: &K, mut entry: Entry<N, V>| -> Entry<N, V> {
            let Entry { lock, vals } = &mut entry;
            out = if lock != &Some(wid) {
                Err(IsPreempted)
            } else {
                *lock = None;
                vals.insert(wid, (val, BTreeSet::new()));
                let pred = vals.range_mut(..wid).last();
                match pred {
                    None => Ok(BTreeSet::new()),
                    Some((_, (_, deps))) =>{
                        let vic = deps.range(wid.succ()..).map(|x| *x);
                        Ok(vic.collect::<BTreeSet<N>>())
                    }
                }
            };
            return entry;
        };
        self.inner.alter(key, upd);
        return out;
    }
    /// set a write lock on this entry, return exempted if there is any
    pub fn wlock(&self, key: K, wid: N)
    -> Result<Option<N>, KVTableErr> {
        let mut out = Ok(None);
        let upd = |entry: &mut Entry<N, V>| {
            let Entry { lock, .. } = entry;
            if lock == &None {
                *lock = Some(wid);
            } else {
                let holder = lock.unwrap();
                if holder > wid {
                    *lock = Some(wid);
                    out = Ok(Some(holder));
                } else if holder == wid {
                    out = Ok(None);
                } else {
                    *lock = Some(wid);
                    out = Err(WouldBlock);
                }
            }
        };
        self.inner.entry(key)
            .and_modify(upd)
            .or_insert(Entry {
                lock: Some(wid),
                vals: BTreeMap::new()
            });
        return out;
    }
    /// prune an entry, only keeps information after cutter id.
    pub fn prune(&self, key: &K, cut: &N) {
        let upd = |_key: &K, mut entry: Entry<N, V>| -> Entry<N, V> {
            entry.vals = entry.vals.split_off(&cut);
            return entry;
        };
        self.inner.alter(key, upd);
        self.inner.remove_if(key, |_, entry| entry.is_empty());
    }
}

use super::error::*;
use crate::utilities::*;
use super::tpool::*;
use super::twrap::*;
use parking_lot::Mutex;
use std::fmt::Debug;
use std::hash::Hash;

use crate::constraint::*;
use crate::rw::*;
use crate::tx::*;
use std::collections::BTreeSet;

// we use this macro to avoid writing the same trait bounds for multiple times
macro_rules! ellipsis_trait_bag {
    ({$T: ty, $V: ty}
    // header like {pub struct ...} {impl ...}
    {$($HeaderBlock: tt)*}
    // the place where 'where clause' will be inserted
    where ...
    // a following content block
    $($ContentBlock: tt)*
    ) => {paste::paste!{
        $($HeaderBlock)*
        where
            $V: Sync + Id + Clone + Debug + 'static,
            $T: Tx<$V> + Sync + Debug + 'static,
            $V::I: Eq + Hash + Sync + Clone + Debug,
            $T::I: Sync + Copy + Ord + Hash + Nat + Debug + 'static,
            $T::Prp: Filter<$V> + MaybeIndexer<$V::I> + Debug,
            $T::Map: Mapper<$V::I, $V> + Debug,
            $T: TxCkpt,
            Ckpt<$T>: Sync,
        $($ContentBlock)*
    }}
}

ellipsis_trait_bag![{T, V}

{pub struct KVSplice<T, V>}
where ...
{
    // the key value table
    table: KVTable<T::I, V::I, V>,
    // pending transactions
    tpool: TPool<KVSpliceTx<V, T>, V>,
    // the transcations that need a roll back
    reset: dashmap::DashSet<T::I>,
    // the checkpoints of a transaction
    ckpts: dashmap::DashMap<T::I, T::Ckpt>,
    // the last committed transaction
    progress: Mutex<T::I>,
    // the last submitted transaction
    last_tid: Mutex<T::I>,
}

];

ellipsis_trait_bag![{T, V}

{impl<T, V> KVSplice<T, V>}
where ...
{
    pub fn new() -> Self {
        Self {
            table: KVTable::new(),
            tpool: TPool::new(),
            reset: dashmap::DashSet::new(),
            ckpts: dashmap::DashMap::new(),
            progress: Mutex::new(T::I::zero()),
            last_tid: Mutex::new(T::I::zero()),
        }
    }
    fn reset(&self, mut txn: KVSpliceTx<V, T>) {
        // ----------------------------------------------
        #[cfg(feature="internal_info")]
        println!(
            "[{:<8?} reset]       at:{:<8?}",
            txn.id(), self.progress()
        );
        // ----------------------------------------------
        let tid = txn.id();
        for (key, (_val, ver)) in &txn.ax.rdset {
            self.table.unread(key, &tid, ver);
        }
        for (key, (_val, ispub)) in &txn.ax.wrset {
            let victim = if *ispub {
                self.table.unwrite(key, &tid)
            } else {
                self.table.unwlock(key, &tid);
                BTreeSet::new()
            };
            for vic in victim.range(self.progress().succ()..) {
                self.reset.insert(*vic);
            }
        }
        let ckpt = self.ckpts.get(&tid).unwrap_or_else(|| unreachable!());
        *txn.ax.as_mut() = Aux::new();
        txn.tx.goto(*ckpt);
        self.tpool.put_todo(txn);
        self.reset.remove(&tid);
    }
    fn submit(&self, tid: T::I) {
        let mut lid = self.last_tid.lock();
        *lid = lid.max(tid);
    }
    fn progress(&self) -> T::I {
        *(self.progress.lock())
    }
    fn get_next(&self) -> Option<KVSpliceTx<V, T>> {
        self.tpool.get_prog(self.progress().succ())
    }
}

];

type MapOf<V, T> = <KVSpliceTx<V, T> as Tx<V>>::Map;
type PrpOf<V, T> = <KVSpliceTx<V, T> as Tx<V>>::Prp;

ellipsis_trait_bag![{T, V}

{impl<T, V, D> RWControl<V, KVSpliceTx<V, T>, D> for KVSplice<T, V>}
where ...
    D: RWDurable<V, KVSpliceTx<V, T>>,
{
    type Err = KVSpliceErr<D::Err>;
    fn rd(&self, mut txn: KVSpliceTx<V, T>, prp: PrpOf<V, T>, dur: &D)
    -> Result<Option<KVSpliceTx<V, T>>, Self::Err> {
        use KVSpliceErr::*;
        use KVTableErr::*;
        assert!(txn.id() != T::I::zero());
        let tid = txn.id();
        if self.reset.contains(&tid) {
            self.reset(txn);
            return Ok(self.get_next())
        }
        let mut map = Vec::new();
        let mut should_read_durable = false;
        if let Some(keys) = prp.tryc_indexer() {
            // read versions in range [progress + 1, tid]
            for key in keys {
                if let Some(val) = txn.ax.read_local(&key) {
                    if val.is_some() { map.push((key, val)) }
                    continue
                }
                // read latest previous version, add tid to dependencies
                match self.table.read(key.clone(), tid) {
                    Ok((val, ver)) => {
                        if val.is_some() { map.push((key.clone(), val.clone())) }
                        txn.ax.rdset.insert(key, (val, ver));
                    },
                    Err(DepDurable) => {
                        should_read_durable = true;
                    },
                    Err(_) => unreachable!()
                }
            }
        } else {
            todo!("non-indexing query type");
        }
        // -------------------------------------------------------------
        #[cfg(feature="internal_info")]
        println!(
            "[{:<8?}    rd]       at:{:<8?}      {:?}    {:?}",
            txn.id(), self.progress(), prp, map);
        // -------------------------------------------------------------
        // fallback to durable storage
        if should_read_durable {
            // read versions in range [0, progress]
            for (key, val) in dur.rd(prp).map_err(External)?.into_mapping() {
                // put them into map iff there is no later version
                if txn.ax.wrset.contains_key(&key) { continue }
                if txn.ax.rdset.contains_key(&key) { continue }
                if val.is_some() { map.push((key.clone(), val.clone())) }
                txn.ax.rdset.insert(key, (val, T::I::zero()));
            }
        }
        Ok(Some(txn.rd(Mapper::from_mapping(map.into_iter()))))
    }
    fn wr(&self, mut txn: KVSpliceTx<V, T>, map: MapOf<V, T>, _dur: &D)
    -> Result<Option<KVSpliceTx<V, T>>, Self::Err> {
        // -------------------------------------------------------------
        #[cfg(feature="internal_info")]
        println!(
            "[{:<8?}    wr]       at:{:<8?}      {:?}",
            txn.id(), self.progress(), map);
        // -------------------------------------------------------------
        assert!(txn.id() != T::I::zero());
        let tid = txn.id();
        if self.reset.contains(&tid) {
            self.reset(txn);
            return Ok(self.get_next())
        }
        for (key, val) in map.into_mapping() {
            // if write lock is already acquired
            if txn.ax.wrset.contains_key(&key) {
                *txn.ax.wrset.get_mut(&key).unwrap() = (val, false);
                continue
            }
            // acquire write lock and write locally
            match self.table.wlock(key.clone(), tid) {
                Ok(Some(preempted)) => {
                    self.reset.insert(preempted);
                    txn.ax.wrset.insert(key, (val, false));
                },
                Ok(None) => {
                    txn.ax.wrset.insert(key, (val, false));
                },
                Err(KVTableErr::WouldBlock) => {
                    self.tpool.put_todo(txn);
                    return Ok(self.get_next())
                },
                Err(_) => unreachable!()
            };
        }
        return Ok(Some(txn.wr()));
    }
    fn done(&self, mut txn: KVSpliceTx<V, T>, end: End, dur: &D)
    -> Result<(Option<KVSpliceTx<V, T>>, Option<Option<T::Out>>), Self::Err> {
        use KVSpliceErr::*;
        use KVTableErr::*;
        // -------------------------------------------------------------
        #[cfg(feature="internal_info")]
        println!(
            "[{:<8?}  done]       at:{:<8?}      {:?}", 
            txn.id(), self.progress(), end);
        // -------------------------------------------------------------
        let tid = txn.id();
        if self.reset.contains(&tid) {
            self.reset(txn);
            return Ok((self.get_next(), None));
        }
        for (key, (val, ispub)) in txn.ax.wrset.iter_mut() {
            if matches!(end, End::Abort) {
                self.table.unwlock(&key, &tid);
                continue
            }
            if txn.ax.wrpub { continue }
            if *ispub { continue }
            let wr_result = self.table.write(key, val.clone(), tid);
            match wr_result {
                Ok(reset) => for r in reset {
                    self.reset.insert(r);
                }
                Err(IsPreempted) => {
                    self.reset(txn);
                    return Ok((self.get_next(), None));
                }
                Err(_) => unreachable!(),
            }
            *ispub = true
        }
        txn.ax.wrpub = true;
        if self.progress().succ() != tid {
            self.tpool.put_done(txn);
            return Ok((self.get_next(), None));
        }
        // -------------------------------------------------------------
        #[cfg(feature="internal_info")]
        println!(
            "[{:<8?} final]       at:{:<8?}      {:?}", 
            txn.id(), self.progress(), end);
        // -------------------------------------------------------------
        if self.reset.contains(&tid) {
            self.reset(txn);
            return Ok((self.get_next(), None));
        }
        let mut map = vec![];
        for (key, (val, _)) in &txn.ax.wrset {
            if matches!(end, End::Abort) { continue }
            map.push((key.clone(), val.clone()));
        }
        dur.wr(&txn, Mapper::from_mapping(map.into_iter()))
            .map_err(External)?;
        for (key, (_val, ispub)) in txn.ax.wrset.drain() {
            if matches!(end, End::Abort) { continue }
            debug_assert!(ispub);
            self.table.prune(&key, &tid);
        }
        {*self.progress.lock() = tid;}
        self.ckpts.remove(&tid);
        Ok((self.get_next(), Some(txn.cl())))
    }
    fn open(&self, txn: &mut KVSpliceTx<V, T>, dur: &D)
    -> Result<(), Self::Err> {
        // -------------------------------------------------------------
        #[cfg(feature="internal_info")]
        println!(
            "[{:<8?}  open]       at:{:<8?}", 
            txn.id(), self.progress());
        // -------------------------------------------------------------
        assert!(txn.id() != T::I::zero());
        let tid = txn.id();
        self.submit(tid);
        self.ckpts.insert(tid, txn.tx.make());
        dur.open(txn).unwrap_or(());
        return Ok(())
    }
}

];

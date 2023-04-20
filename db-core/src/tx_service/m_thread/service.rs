use paste::paste;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::collections::BTreeMap;
use flume::{Receiver, Sender};
use typing::tx::*;
use typing::rw::*;
use super::error::*;
use super::MThreadHandle;

// we use this macro to avoid writing the same trait bounds for multiple times
macro_rules! ellipsis_trait_bag {
    ({$T: ty, $V: ty, $Dur: ty, $Con: ty, $InnerT: ty}
    $(// repeated unit
        // header like {pub struct ...} {impl ...}
        {$($HeaderBlock: tt)*}
        // the place where 'where clause' will be inserted
            where ...
        // a following content block
        $ContentBlock: tt
    )*
    ) => {paste!{$(
        $($HeaderBlock)*
        where
            // transaction properties
            $T: Debug + Tx<$V> + Send + 'static,
            $T::I: Ord + Sync + std::hash::Hash + Send + Debug + 'static,
            $InnerT: Debug + Tx<$V, I = $T::I, Prp = $T::Prp, Map = $T::Map, Out = $T::Out>,
            $InnerT: 'static,
            $T::Out: Sync + Send + 'static,
            // required properties of internal components
            $Dur: RWDurable<$V, $InnerT> + Sync + Send + 'static,
            $Con: RWControl<$V, $InnerT, $Dur> + Sync + Send + 'static,
            // require dur and con errors to have proper debug format
            $Dur::Err: std::fmt::Debug,
            $Con::Err: std::fmt::Debug,
            // internally used transaction type
            $InnerT: Tx<$V>
        $ContentBlock
    )*}}
}

ellipsis_trait_bag! {{T, V, Dur, Con, InnerT}

{pub struct MThreadService<T, V, Dur, Con, InnerT>}
where ...
{
    // concurrency and durablility control module
    con: Arc<Con>,
    dur: Arc<Dur>,

    // input wrapper function
    wrapper: fn(T) -> InnerT,

    // init configuration
    workers: usize, // number of workers

    // information for each worker thread
    sender_aggr: Option<Sender<T>>,
    killer_list: Vec<Arc<AtomicBool>>,
    worker_list: Vec<JoinHandle<()>>,

    // registered output
    output_list: Arc<dashmap::DashMap<T::I, Option<T::Out>>>,

    // internal transaction type
    inner_transaction_marker: PhantomData<InnerT>,
}
}


ellipsis_trait_bag!{{T, V, Dur, Con, InnerT}

{impl<T, V, Dur, Con, InnerT> Drop for MThreadService<T, V, Dur, Con, InnerT>}
where ...
{
    fn drop(&mut self) {
        if self.worker_list.len() != 0 {
            let error_report = self.close_all();
            assert!(error_report.len() == 0);
        }
    }
}

}

ellipsis_trait_bag!{{T, V, Dur, Con, InnerT}

{impl<T, V, Dur, Con, InnerT> MThreadService<T, V, Dur, Con, InnerT>}
where ...
{
    pub fn new(workers: usize, wrapper: fn(T) -> InnerT, con: Con, dur: Dur) -> Self {
        Self {
            workers,
            wrapper,
            sender_aggr: None,
            killer_list: vec![],
            worker_list: vec![],
            output_list: Arc::new(dashmap::DashMap::new()),
            con: Arc::new(con),
            dur: Arc::new(dur),
            inner_transaction_marker: PhantomData,
        }
    }
    pub fn get_handle(&self) -> Result<MThreadHandle<T, V>, MThreadServiceError<T>> {
        let sender_aggr = (&self.sender_aggr).as_ref().ok_or(MThreadServiceError::NoSenderOpen)?.clone();
        Ok(MThreadHandle {sender_aggr, output_list: Arc::clone(&self.output_list)})
    }
    fn start_all(&mut self) {
        let (send, recv) = flume::bounded(self.workers+4);
        self.sender_aggr = Some(send);
        // start all workers, store killer flags in killer list
        for i in 0..self.workers {
            #[cfg(feature="internal_info")]
            println!("starting worker [{i}]");
            // start one worker with given parameter
            let (killer, worker) = self.start_one(i, recv.clone());
            self.killer_list.push(killer);
            self.worker_list.push(worker);
        }
        // the worker number and killer number should be exactly as requested.
        assert!(self.worker_list.len() == self.workers);
        assert!(self.killer_list.len() == self.workers);
    }
    fn close_all(&mut self) -> Vec<Box<dyn std::any::Any + Send>> {
        use std::sync::atomic::Ordering::*;
        // the worker number and killer number should be exactly as requested.
        assert!(self.worker_list.len() == self.workers);
        assert!(self.killer_list.len() == self.workers);
        self.sender_aggr = None;
        // close all workers, collect error reports into error collection
        let mut error_collection = vec![];
        for _i in 0..self.workers {
            #[cfg(feature="internal_info")]
            println!("closing worker [{_i}]");
            self.killer_list.pop().unwrap().store(true, Relaxed);
            match self.worker_list.pop().unwrap().join() {
                Ok(()) => (),
                Err(e) => error_collection.push(e),
            };
        }
        // return error collection
        return error_collection;
    }
    /// process one transaction (txn) with
    ///     a durability controller (dur),
    ///     a concurrency controller (con), and
    ///     finally an output list (ols) to write results to.
    fn handle_tx(
        txn: InnerT,
        dur: &Dur,
        con: &Con,
        ols: &dashmap::DashMap<T::I, Option<T::Out>>,
    ) -> Option<InnerT> {
        use RWClosure::*;
        // record transaction id by copying
        let tid = txn.id();
        // dispatch transaction closures to read-write control interfaces
        match txn.go() {
            // move a transaction forward with an internal operation
            Op(txn) => Some(txn.op()),
            // handle read requests, panic on any internal component error
            Rd(txn, prp) => con.rd(txn, prp, dur).unwrap(),
            // handle write requests, panic on any internal component error
            Wr(txn, map) => con.wr(txn, map, dur).unwrap(),
            // close transaction with a given ending
            Cl(txn, end) => {
                // get the next transaction and output through concurrency control module
                let (txn, out) = con.done(txn, end, dur).unwrap();
                // register output if there is some
                match out {
                    // if output is some, insert it into output_list.
                    Some(out) => {
                        ols.insert(tid, out);
                    }
                    // if output is none, the transaction is not really done.
                    None => {}
                };
                // return the transaction
                return txn;
            }
        }
    }
    /// start one worker
    fn start_one(&self, _i: usize, recv_handle: Receiver<T>) -> (Arc<AtomicBool>, JoinHandle<()>) {
        // initialize resources
        let con = Arc::clone(&self.con);
        let dur = Arc::clone(&self.dur);
        let ols = Arc::clone(&self.output_list);
        let sigterm = Arc::new(AtomicBool::new(false));
        let cpyterm = Arc::clone(&sigterm);
        let wrapper = self.wrapper;
        // the inner worker function inside a thread
        let worker_fn = move || {
            use std::sync::atomic::Ordering::*;
            // a local pool for transactions
            let mut pooling = BTreeMap::new();
            let core_ls = core_affinity::get_core_ids().unwrap();
            let core_id = core_ls[(_i+1) % core_ls.len()];
            core_affinity::set_for_current(core_id);
            while !sigterm.load(Relaxed) {
                if rand::random::<usize>() % (pooling.len() + 1) == 0 {
                    let mut txn = match recv_handle.recv() {
                        Ok(txn) => (wrapper)(txn),
                        Err(_) => continue,
                    };
                    #[cfg(feature="internal_info")]
                    println!("receive transaction [{:?}]\n{:?}", txn.id(), pooling.iter().map(|(id, _)| id).collect::<Vec<_>>());
                    con.open(&mut txn, &dur).unwrap();
                    pooling.insert(txn.id(), txn);
                }
                let mut txn = match pooling.pop_first() {
                    Some((_, txn)) => txn,
                    None => continue,
                };
                loop {
                    // handle transaction with predefined handler
                    txn = match Self::handle_tx(txn, &dur, &con, &ols) {
                        Some(txn) => txn,
                        None => break,
                    };
                }
            }
        };
        // return (kill signal handle, transaction sender handle, worker thread join handle)
        (cpyterm, std::thread::spawn(worker_fn))
    }
}

}


ellipsis_trait_bag!{{T, V, Dur, Con, InnerT}

{impl<T, V, Dur, Con, InnerT> TxService<T, V> for MThreadService<T, V, Dur, Con, InnerT>}
where ...
{
    type Err = MThreadServiceError<T>;
    fn close(&mut self) -> Result<(), Self::Err> {
        match self.close_all() {
            report if report.is_empty() => Ok(()),
            report => Err(MThreadServiceError::ShutdownErrorReport(report))
        }
    }
    fn start(&mut self) -> Result<(), Self::Err> {
        Ok(self.start_all())
    }
    fn get(&self, i: T::I) -> Result<Option<T::Out>, Self::Err> {
        match self.output_list.remove(&i) {
            Some((_, x)) => Ok(x),
            None => Err(MThreadServiceError::TxPending),
        }
    }
    fn put(&self, t: T) -> Result<(), Self::Err> {
        use flume::SendError;
        let s = (&self.sender_aggr).as_ref().ok_or(MThreadServiceError::NoSenderOpen)?;
        s.send(t).map_err(|SendError(t)| MThreadServiceError::SendError(t))
    }
}

}
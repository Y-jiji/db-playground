use crate::rw::*;
use crate::tx::*;

pub struct Null<V, T>(std::marker::PhantomData<(V, T)>);

impl<V, T> Null<V, T> {
    pub fn new() -> Self { Null(std::marker::PhantomData) }
}

impl<V, T, D> RWControl<V, T, D> for Null<V, T>
where
    D: RWDurable<V, T>,
    D::Err: std::fmt::Debug,
    T: Tx<V>,
{
    type Err = String;
    fn done(&self, txn: T, end: End, dur: &D) -> Result<(Option<T>, Option<Option<T::Out>>), Self::Err> {
        match dur.done(&txn, end) {
            Err(e) => Err(format!("{e:?}")),
            Ok(()) => Ok((None, Some(txn.cl())))
        }
    }
    fn open(&self, txn: &mut T, dur: &D) -> Result<(), Self::Err> {
        match dur.open(&txn) {
            Err(e) => Err(format!("{e:?}")),
            Ok(()) => Ok(())
        }
    }
    fn rd(&self, txn: T, prp: T::Prp, dur: &D) -> Result<Option<T>, Self::Err> {
        let map = dur.rd(prp).map_err(|e| format!("{e:?}"))?;
        Ok(Some(txn.rd(map)))
    }
    fn wr(&self, txn: T, map: T::Map, dur: &D) -> Result<Option<T>, Self::Err> {
        dur.wr(&txn, map).map_err(|e| format!("{e:?}"))?;
        Ok(Some(txn.wr()))
    }
}
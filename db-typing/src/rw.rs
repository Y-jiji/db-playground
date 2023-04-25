//! ## rw
//! read-write control interface traits

use crate::tx::*;

// guarantee the durability of reads and writes
pub trait RWDurable<V, T: Tx<V>> {
    type Err;
    fn rd(&self, prp: T::Prp) -> Result<T::Map, Self::Err>;
    fn wr(&self, txn: &T, map: T::Map) -> Result<(), Self::Err>;
    fn open(&self, txn: &T) -> Result<(), Self::Err>;
    fn done(&self, txn: &T, end: End) -> Result<(), Self::Err>;
}

// a read-write control inteface
pub trait RWControl<V, T: Tx<V>, D: RWDurable<V, T>> {
    type Err;
    fn rd(&self, txn: T, prp: T::Prp, dur: &D) -> Result<Option<T>, Self::Err>;
    fn wr(&self, txn: T, map: T::Map, dur: &D) -> Result<Option<T>, Self::Err>;
    fn open(&self, txn: T, dur: &D) -> Result<T, Self::Err>;
    fn done(&self, txn: T, end: End, dur: &D) -> Result<(Option<T>, Option<Option<T::Out>>), Self::Err>;
}
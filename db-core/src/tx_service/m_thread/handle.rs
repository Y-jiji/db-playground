use flume::Sender;
use typing::tx::TxService;
use std::fmt::Debug;
use typing::tx::*;
use std::sync::Arc;
use super::error::*;

// a handle of m_thread service
pub struct MThreadHandle<T, V> 
where
    T: Debug + Tx<V> + Send + 'static,
    T::I: Ord + Sync + std::hash::Hash + Send + Debug + 'static,
    T::Out: Sync + Send + 'static,
{
    pub(super) sender_aggr: Sender<T>,
    pub(super) output_list: Arc<dashmap::DashMap<T::I, Option<T::Out>>>,
}

impl<T, V> TxService<T, V> for MThreadHandle<T, V> 
where
    T: Debug + Tx<V> + Send + 'static,
    T::I: Ord + Sync + std::hash::Hash + Send + Debug + 'static,
    T::Out: Sync + Send + 'static,
{
    type Err = MThreadServiceError<T>;
    fn close(&mut self) -> Result<(), Self::Err> {
        Ok(())
    }
    fn start(&mut self) -> Result<(), Self::Err> {
        Ok(())
    }
    fn get(&self, i: T::I) -> Result<Option<T::Out>, Self::Err> {
        match self.output_list.remove(&i) {
            Some((_, x)) => Ok(x),
            None => Err(MThreadServiceError::TxPending),
        }
    }
    fn put(&self, t: T) -> Result<(), Self::Err> {
        use flume::SendError;
        self.sender_aggr.send(t).map_err(|SendError(t)| MThreadServiceError::SendError(t))
    }
}
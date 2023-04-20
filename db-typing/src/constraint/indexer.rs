pub trait MaybeIndexer<I> {
    fn tryc_indexer(&self) -> Option<Box<dyn Iterator<Item = I> + '_>> {
        None
    }
    fn from_indexer<Iter: Iterator<Item = I>>(iter: Iter) -> Self;
}
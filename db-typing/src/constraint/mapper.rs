/// collect trait on map
pub trait MaybeMapper<I, V> {
    /// mapper to iterator
    fn tryc_mapping(&mut self) -> Option<Box<dyn Iterator<Item = (I, Option<V>)> + '_>> {
        None
    }
    /// from iterator to mapper
    fn from_mapping<Iter: Iterator<Item = (I, Option<V>)>>(iter: Iter) -> Self;
}

pub trait Mapper<I, V> {
    /// mapper to iterator
    fn into_mapping(&self) -> Box<dyn Iterator<Item = (I, Option<V>)> + '_>;
    /// from iterator to mapper
    fn from_mapping<Iter: Iterator<Item = (I, Option<V>)>>(iter: Iter) -> Self;
}

impl<I, V, T> MaybeMapper<I, V> for T
where
    T: Mapper<I, V>,
{
    fn tryc_mapping(&mut self) -> Option<Box<dyn Iterator<Item = (I, Option<V>)> + '_>> {
        Some(self.into_mapping())
    }
    fn from_mapping<Iter: Iterator<Item = (I, Option<V>)>>(iter: Iter) -> Self {
        Mapper::<I, V>::from_mapping(iter)
    }
}

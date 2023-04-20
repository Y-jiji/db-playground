/// filter trait on proposition
pub trait MaybeFilter<V> {
    /// convert itself to a filter which borrows self
    fn tryc_filter(&self) -> Option<Box<dyn Fn(&V) -> bool + '_>> {
        None
    }
}

pub trait Filter<V> {
    /// convert itself to a filter which borrows self
    fn into_filter(&self) -> Box<dyn Fn(&V) -> bool + '_>;
}

impl<V, T: Filter<V>> MaybeFilter<V> for T {
    fn tryc_filter(&self) -> Option<Box<dyn Fn(&V) -> bool + '_>> {
        Some(self.into_filter())
    }
}

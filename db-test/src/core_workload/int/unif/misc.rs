use typing::constraint::*;

#[derive(Debug, Clone, Copy)]
pub struct U64Tup(pub u64, pub u64);

#[derive(Debug, Clone, Copy)]
pub struct U64Map(pub Option<(u64, Option<U64Tup>)>);

#[derive(Debug, Clone, Copy)]
pub struct U64Prp(pub u64);

impl Id for U64Tup {
    type I = u64;
    fn id(&self) -> Self::I {
        self.0
    }
}

impl Mapper<u64, U64Tup> for U64Map {
    fn from_mapping<Iter: Iterator<Item = (u64, Option<U64Tup>)>>(mut iter: Iter) -> Self {
        return U64Map(iter.next());
    }
    fn into_mapping(&self) -> Box<dyn Iterator<Item = (u64, Option<U64Tup>)> + '_> {
        let mut twice = false;
        let this = *self;
        return Box::new(std::iter::from_fn(move || {
            if twice {
                None
            } else {
                twice = true;
                this.0
            }
        }))
    }
}

impl Filter<U64Tup> for U64Prp {
    fn into_filter(&self) -> Box<dyn Fn(&U64Tup) -> bool + '_> {
        Box::new(|U64Tup(k, _)| *k == self.0)
    }
} 

impl MaybeIndexer<u64> for U64Prp {
    fn from_indexer<Iter: Iterator<Item = u64>>(mut iter: Iter) -> Self {
        U64Prp(iter.next().unwrap())
    }
    fn tryc_indexer(&self) -> Option<Box<dyn Iterator<Item = u64> + '_>> {
        let mut twice = false;
        let U64Prp(a) = *self;
        Some(Box::new(std::iter::from_fn(move || {
            if twice {
                None
            } else {
                twice = true;
                Some(a)
            }
        })))
    }
}
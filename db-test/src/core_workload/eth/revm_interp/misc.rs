use revm_primitives::*;
use typing::constraint::{Filter, Mapper, MaybeIndexer, Id};

#[derive(Debug, Clone)]
pub struct EVMU256Tup(pub U256, pub U256);

#[derive(Debug, Clone)]
pub struct EVMU256Prp(pub U256);

#[derive(Debug, Clone)]
pub struct EVMU256Map(pub Option<(U256, Option<U256>)>);

impl Id for EVMU256Tup {
    type I = U256;
    fn id(&self) -> Self::I {
        self.0  
    }
}

impl Mapper<U256, EVMU256Tup> for EVMU256Map {
    fn from_mapping<Iter: Iterator<Item = (U256, Option<EVMU256Tup>)>>(mut iter: Iter) -> Self {
        EVMU256Map(match iter.next() {
            None => None,
            Some((key, None)) => Some((key, None)),
            Some((key, Some(EVMU256Tup(_key, val)))) => Some((key, Some(val))),
        })
    }
    fn into_mapping(&self) -> Box<dyn Iterator<Item = (U256, Option<EVMU256Tup>)> + '_> {
        let mut is_twice = false;
        let inner_fn = move || {
            if is_twice { return None }
            is_twice = true;
            match self.0 {
                Some((key, Some(val))) => Some((key, Some(EVMU256Tup(key, val)))),
                Some((_key, None)) => None,
                None => None,
            }
        };
        Box::new(std::iter::from_fn(inner_fn))
    }
}

impl MaybeIndexer<U256> for EVMU256Prp {
    fn from_indexer<Iter: Iterator<Item = U256>>(mut iter: Iter) -> Self {
        EVMU256Prp(iter.next().unwrap())
    }
    fn tryc_indexer(&self) -> Option<Box<dyn Iterator<Item = U256> + '_>> {
        let mut is_twice = false;
        let inner_fn = move || {
            if is_twice { return None }
            is_twice = true;
            Some(self.0)
        };
        Some(Box::new(std::iter::from_fn(inner_fn)))
    }
}

impl Filter<EVMU256Tup> for EVMU256Prp {
    fn into_filter(&self) -> Box<dyn Fn(&EVMU256Tup) -> bool + '_> {
        Box::new(|x| &self.0 == &x.0)
    }
}
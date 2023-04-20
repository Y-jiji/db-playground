use crate::tx::*;

// pack a transaction auxiliary information
#[derive(Debug, Clone)]
pub struct Wrap<T, A> {
    pub tx: T, // transaction
    pub ax: A, // auxiliary information
}

impl<T, V, A> Tx<V> for Wrap<T, A>
where
    T: Tx<V>,
{
    type I = T::I;
    fn id(&self) -> T::I {
        self.tx.id()
    }

    type Prp = T::Prp;
    type Map = T::Map;
    type Out = T::Out;
    fn go(self) -> RWClosure<Self, Self::Prp, Self::Map> {
        use RWClosure::*;
        let Wrap { tx, ax } = self;
        match tx.go() {
            Rd(tx, prp) => Rd(Wrap { tx, ax }, prp),
            Wr(tx, map) => Wr(Wrap { tx, ax }, map),
            Cl(tx, end) => Cl(Wrap { tx, ax }, end),
            Op(tx) => Op(Wrap { tx, ax }),
        }
    }
    fn op(mut self) -> Self {
        self.tx = self.tx.op();
        self
    }
    fn rd(mut self, map: T::Map) -> Self {
        self.tx = self.tx.rd(map);
        self
    }
    fn wr(mut self) -> Self {
        self.tx = self.tx.wr();
        self
    }
    fn cl(self) -> Option<T::Out> {
        self.tx.cl()
    }
}
use typing::tx::*;
use typing::constraint::*;
use super::*;

// transaction that reads and writes interger values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct U64Txn {
    // the id of u64
    pub(super) id: u64,
    // seed of randomization
    pub(super) seed: u64,
    // the proportion of four operations: read(r), write(w), abort(a) and commit(c)
    pub(super) rwac: (u64, u64, u64, u64),
    // the upper bound of value
    pub(super) vrng: u64,
    // the mask of value
    pub(super) vmsk: u64,
    // the offset of value
    pub(super) voff: u64,
    // the counts of read and write ops
    pub(super) rcnt: u64,
    pub(super) wcnt: u64,
}

impl TxCkpt for U64Txn {
    type Ckpt = u64;
    fn goto(&mut self, ckpt: Self::Ckpt) {
        self.seed = ckpt;
    }
    fn make(&mut self) -> Self::Ckpt {
        self.seed
    }
}

impl U64Txn {
    // get the random number
    fn num(&mut self) -> u64 {
        let mut s0 = (self.seed >> u32::BITS & u32::MAX as u64) as u32;
        let mut s1 = (self.seed & u32::MAX as u64) as u32;
        s1 ^= s0;
        s0 = s0.rotate_left(26) ^ s1 ^ (s1 << 9);
        s1 = s1.rotate_left(13);
        self.seed = ((s0 as u64) << (u32::BITS as u64)) | (s1 as u64);
        return self.seed;
    }
    // get a value inside the range
    fn val(&mut self) -> u64 {
        let mut s0 = (self.seed >> u32::BITS & u32::MAX as u64) as u32;
        let mut s1 = (self.seed & u32::MAX as u64) as u32;
        s1 ^= s0;
        s0 = s0.rotate_left(26) ^ s1 ^ (s1 << 9);
        s1 = s1.rotate_left(13);
        self.seed = ((s0 as u64) << (u32::BITS as u64)) | (s1 as u64);
        return ((self.seed & self.vmsk) | self.voff) % self.vrng;
    }
}

impl Tx<U64Tup> for U64Txn {
    type I = u64;
    type Prp = U64Prp;
    type Map = U64Map;
    type Out = u64;
    fn id(&self) -> Self::I { self.id }
    fn go(mut self) -> RWClosure<Self, Self::Prp, Self::Map> {
        let seed = self.seed;
        let rwac = self.num() % (2*self.rwac.3);
        if rwac < self.rwac.0 {
            let k = self.val();
            self.seed = seed;
            return RWClosure::Rd(self, U64Prp(k));
        }
        if rwac < self.rwac.1 {
            let k = self.val();
            let v = if self.num() & 1 == 0 { None } else { Some(U64Tup(k, self.val())) };
            self.seed = seed;
            return RWClosure::Wr(self, U64Map(Some((k, v))));
        }
        if rwac < self.rwac.2 {
            self.seed = seed;
            return RWClosure::Cl(self, End::Abort);
        }
        if rwac < self.rwac.3 {
            self.seed = seed;
            return RWClosure::Cl(self, End::Ready);
        }
        return RWClosure::Op(self);
    }
    fn op(mut self) -> Self {
        for _ in 0..(1<<8) { self.num(); }
        self
    }
    fn rd(mut self, map: Self::Map) -> Self {
        self.rcnt += 1;
        if let U64Map(Some((_, v))) = map {
            self.num();
            let read = v.expect("map should be null if there is no such value").1 as u128;
            let seed = self.seed as u128;
            self.seed = (read + seed & 0xffffffffffffffff) as u64;
            self
        } else {
            self.num();
            self.num();
            self.num();
            self.num();
            self
        }
    }
    fn wr(mut self) -> Self {
        self.wcnt += 1;
        self.num();
        self.num();
        self.num();
        self.num();
        self.num();
        self
    }
    fn cl(mut self) -> Option<Self::Out> {
        let seed = self.seed;
        let rwac = self.num() % self.rwac.3;
        if rwac < self.rwac.1 { 
            panic!("For U64Txn, transaction close must be called after transaction release enumeration option Cl, but you are not actually doing so. ")
        }
        if rwac < self.rwac.2 {
            return None;
        }
        if rwac < self.rwac.3 {
            return Some(seed);
        }
        unreachable!()
    }
}

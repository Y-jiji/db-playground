use super::*;

/// replayable integral read and write worload
///      using Blum Blum Shub pseudo random
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct U64Gen {
    /// seed of random
    seed: u64,
    /// the proportion of read, write, abort and commit
    rwac: (u64, u64, u64, u64),
    /// the upper bound of value
    vrng: u64,
    /// auto-incremental counting for id generation
    incr: u64,
}

impl U64Gen {
    pub fn new(seed: u64, rwac: (u64, u64, u64, u64), vrng: u64) -> Self {
        let rwac = (
            rwac.0,
            rwac.0 + rwac.1,
            rwac.0 + rwac.1 + rwac.2,
            rwac.0 + rwac.1 + rwac.2 + rwac.3,
        );
        U64Gen {
            seed,
            rwac,
            vrng,
            incr: 0,
        }
    }
    // get a transaction
    pub fn get(&mut self) -> U64Txn {
        let id = self.incr;
        self.incr += 1;
        let vmsk = self.num();
        U64Txn {
            id,
            seed: self.num(),
            rwac: self.rwac,
            vrng: self.vrng,
            voff: self.num() & !vmsk,
            vmsk, rcnt: 0, wcnt: 0,
        }
    }
    // get a random number and walk to next number
    fn num(&mut self) -> u64 {
        let mut s0 = (self.seed >> u32::BITS & u32::MAX as u64) as u32;
        let mut s1 = (self.seed & u32::MAX as u64) as u32;
        s1 ^= s0;
        s0 = s0.rotate_left(26) ^ s1 ^ (s1 << 9);
        s1 = s1.rotate_left(13);
        self.seed = ((s0 as u64) << (u32::BITS as u64)) | (s1 as u64);
        return self.seed;
    }
}

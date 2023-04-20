use revm_interpreter::BytecodeLocked;
use revm_primitives::Bytes;
use rand_xoshiro::Xoroshiro64StarStar;
use rand::{SeedableRng, RngCore};
use super::*;

pub struct REVMInterpGen {
    step_len: usize,
    counting: usize,
    bytecode: BytecodeLocked,
    template: fn(&mut Xoroshiro64StarStar) -> Bytes,
    fake_rng: Xoroshiro64StarStar,
}

impl REVMInterpGen {
    /// bytecode: the smart contract byte code
    /// template: the input data template, returns bytes from a given seed
    /// seed: random seed, source of entropy
    pub fn new(
        bytecode: BytecodeLocked, 
        template: fn(&mut Xoroshiro64StarStar) -> Bytes, 
        seed: [u8; 8]
    ) -> REVMInterpGen {
        let fake_rng = Xoroshiro64StarStar::from_seed(seed);
        let counting = 1usize;
        let step_len = 1usize;
        REVMInterpGen { step_len, counting, fake_rng, bytecode, template }
    }
    pub fn new_many(
        bytecode: BytecodeLocked,
        template: fn(&mut Xoroshiro64StarStar) -> Bytes,
        gen_seed: [u8; 8],
        step_len: usize,
    ) -> Vec<REVMInterpGen> {
        // genesis random number generator
        let mut genesis = Xoroshiro64StarStar::from_seed(gen_seed);
        // create new evm instance generator
        (1..=step_len).map(|counting| {
            let mut seed = [0u8; 8];
            genesis.fill_bytes(&mut seed);
            REVMInterpGen {
                counting, step_len, 
                bytecode: bytecode.clone(), 
                template: template.clone(),
                fake_rng: Xoroshiro64StarStar::from_seed(seed)
            }
        }).collect::<Vec<_>>()
    }
    pub fn get(&mut self) -> REVMInterpTxn {
        let count = self.counting;
        let input = (self.template)(&mut self.fake_rng);
        let bytecode = self.bytecode.clone();
        self.counting += self.step_len;
        REVMInterpTxn(Box::new(REVMInterpTxnInner::new(count, bytecode, input)))
    }
}
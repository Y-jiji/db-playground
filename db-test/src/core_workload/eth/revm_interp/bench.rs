use super::*;
use revm_interpreter::{BytecodeLocked, analysis::to_analysed};
use revm_primitives::{Bytes, hex, Bytecode, U256};
use rand::{RngCore, SeedableRng, Rng};
use typing::tx::*;
use std::time::SystemTime;
use rand_xoshiro::Xoroshiro64StarStar;

const BYTECODE_TEN_RW: &str = "608060405234801561001057600080fd5b50600436106100415760003560e01c806340cb7660146100465780639507d39a14610062578063a5843f0814610092575b600080fd5b610060600480360381019061005b9190610268565b6100ae565b005b61007c600480360381019061007791906102f5565b6101f6565b6040516100899190610331565b60405180910390f35b6100ac60048036038101906100a7919061034c565b610212565b005b600081600080898152602001908152602001600020546100ce91906103bb565b6100d891906103bb565b600080888152602001908152602001600020819055506001816000808881526020019081526020016000205461010e91906103bb565b61011891906103bb565b600080878152602001908152602001600020819055506002816000808781526020019081526020016000205461014e91906103bb565b61015891906103bb565b600080868152602001908152602001600020819055506003816000808681526020019081526020016000205461018e91906103bb565b61019891906103bb565b60008085815260200190815260200160002081905550600481600080858152602001908152602001600020546101ce91906103bb565b6101d891906103bb565b60008084815260200190815260200160002081905550505050505050565b6000806000838152602001908152602001600020549050919050565b80600080848152602001908152602001600020819055505050565b600080fd5b6000819050919050565b61024581610232565b811461025057600080fd5b50565b6000813590506102628161023c565b92915050565b60008060008060008060c087890312156102855761028461022d565b5b600061029389828a01610253565b96505060206102a489828a01610253565b95505060406102b589828a01610253565b94505060606102c689828a01610253565b93505060806102d789828a01610253565b92505060a06102e889828a01610253565b9150509295509295509295565b60006020828403121561030b5761030a61022d565b5b600061031984828501610253565b91505092915050565b61032b81610232565b82525050565b60006020820190506103466000830184610322565b92915050565b600080604083850312156103635761036261022d565b5b600061037185828601610253565b925050602061038285828601610253565b9150509250929050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b60006103c682610232565b91506103d183610232565b92508282019050808211156103e9576103e861038c565b5b9291505056fea2646970667358221220c127789972496be42e82ef6dafc3184aee8a4594c6aeb94e5eec4af5566759f064736f6c63430008120033";

#[test]
fn is_txn_fast() {
    debug_assert!(false, "running benchmark in low optimiation level!");
    let seed = [0xea; 8];
    let bytecode: Bytes = hex::decode(BYTECODE_TEN_RW).unwrap().into();
    let bytecode = to_analysed(Bytecode::new_raw(bytecode));
    let bytecode = BytecodeLocked::try_from(bytecode).unwrap();
    fn template(rng: &mut Xoroshiro64StarStar) -> Bytes {
        let mut ten_key_and_val = vec![0u8; 11*64];
        rng.fill_bytes(&mut ten_key_and_val);
        let prefix = hex::decode("40cb7660").unwrap();
        [prefix, ten_key_and_val].concat().into()
    }
    let mut gen = REVMInterpGen::new(bytecode, template, seed);
    let mut rng = Xoroshiro64StarStar::from_seed([0x7b; 8]);
    let mut cnt = 0usize;
    let start = SystemTime::now();
    for _ in 0..40000 {
        let mut txn = gen.get();
        loop {
            // this is string is super long
            // making this check as slow as a sloth
            // however, since we don't have eq trait on txn, we have to do this. 
            txn = match txn.go() {
                RWClosure::Op(txn) => {
                    txn.op()
                }
                RWClosure::Cl(_txn, _end) => {
                    break
                }
                RWClosure::Rd(txn, EVMU256Prp(key)) => {
                    let val = U256::from_limbs(rng.gen::<[u64; 4]>());
                    txn.rd(EVMU256Map(Some((key, Some(val)))))
                }
                RWClosure::Wr(txn, _map) => {
                    txn.wr()
                }
            };
            cnt += 1;
        }
    }
    println!("revm overall throughput: {:.4} (ops/sec)", 40000.0 / (start.elapsed().unwrap().as_nanos() as f32) * 1000_000_000.0);
    println!("revm txn throughput: {:.4} (ops/sec)", (cnt as f32) / (start.elapsed().unwrap().as_nanos() as f32) * 1000_000_000.0);
}

#[test]
fn is_gen_fast() {
    debug_assert!(false, "running benchmark in low optimiation level!");
    let seed = [0xea; 8];
    let bytecode: Bytes = hex::decode(BYTECODE_TEN_RW).unwrap().into();
    let bytecode = to_analysed(Bytecode::new_raw(bytecode));
    let bytecode = BytecodeLocked::try_from(bytecode).unwrap();
    fn template(rng: &mut Xoroshiro64StarStar) -> Bytes {
        let mut ten_key_and_val = vec![0u8; 11*64];
        rng.fill_bytes(&mut ten_key_and_val);
        let prefix = hex::decode("40cb7660").unwrap();
        [prefix, ten_key_and_val].concat().into()
    }
    let mut gen = REVMInterpGen::new(bytecode, template, seed);
    let mut vec = vec![];
    let start = SystemTime::now();
    for _ in 0..25000 {
        vec.push(gen.get());
    }
    println!("revm gen throughput: {:.4} (txn/sec)", (25000 as f32) / (start.elapsed().unwrap().as_nanos() as f32) * 1000_000_000.0);
}
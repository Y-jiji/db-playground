use super::*;
use typing::tx::*;
use rand::random;
use std::time::*;

#[test]
fn is_txn_fast() {
    debug_assert!(false, "running benchmark in low optimiation level!");
    let seed = random::<u64>();
    let rwac = (100, 100, 1, 1);
    let vrng = 10000u64;
    let mut u64gen = U64Gen::new(seed, rwac, vrng);
    let mut opscnt = 0usize;
    let now = SystemTime::now();
    while now.elapsed().unwrap().as_secs_f32() <= 1.0+1e-18 {
        let mut u64txn = u64gen.get();
        loop {
            u64txn = match u64txn.go() {
                RWClosure::Rd(txn, _) => txn.rd(U64Map(Some((0, Some(U64Tup(0, 0)))))),
                RWClosure::Wr(txn, _) => txn.wr(),
                RWClosure::Cl(_, _) => break,
                RWClosure::Op(txn) => txn.op(),
            };
            opscnt += 1;
        }
    }
    assert!(
        opscnt >= 2_000_000,
        "uniformly generated u64 transaction doesn't run fast enough! only {opscnt}"
    );
}

#[test]
fn is_gen_fast() {
    debug_assert!(false, "running benchmark in low optimiation level!");
    let seed = random::<u64>();
    let rwac = (100, 100, 1, 1);
    let vrng = 10000u64;
    let mut u64gen = U64Gen::new(seed, rwac, vrng);
    let mut gencnt = 0usize;
    let now = SystemTime::now();
    let mut vec = vec![];
    while now.elapsed().unwrap().as_secs_f32() <= 1.0+1e-18 {
        let u64txn = u64gen.get();
        vec.push(u64txn);
        gencnt += 1;
    }
    println!("{gencnt}");
    assert!(
        gencnt >= 2_000_000,
        "uniformly generated u64 transaction doesn't run fast enough! only {gencnt}"
    );
}
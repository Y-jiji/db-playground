use super::*;
use rand::random;
use std::collections::HashSet;
use std::time::*;
use typing::tx::*;

#[test]
fn is_random() {
    let seed = random::<u64>();
    let rwac = (
        random::<u64>() % 10000,
        random::<u64>() % 10000,
        random::<u64>() % 10000,
        random::<u64>() % 10000,
    );
    let vrng = 10000u64;
    let mut holder = HashSet::new();
    let mut u64gen = U64Gen::new(seed, rwac, vrng);
    for _ in 0..10000 {
        assert!(holder.insert(u64gen.get()), "bad! repeated transaction. ");
    }
}

#[test]
fn is_replayable() {
    let seed = random::<u64>();
    let rwac = (
        random::<u64>() % 10000,
        random::<u64>() % 10000,
        random::<u64>() % 10000,
        random::<u64>() % 10000,
    );
    let vrng = 10000u64;
    let mut u64gen_1 = U64Gen::new(seed, rwac, vrng);
    let mut u64gen_2 = U64Gen::new(seed, rwac, vrng);
    for _ in 0..10000 {
        assert!(u64gen_1.get() == u64gen_2.get());
    }
}

#[test]
fn is_stuck() {
    let seed = random::<u64>();
    let rwac = (
        random::<u64>() % 10000,
        random::<u64>() % 10000,
        random::<u64>() % 10000,
        random::<u64>() % 10000,
    );
    let now = SystemTime::now();
    let vrng = 10000u64;
    let mut u64gen = U64Gen::new(seed, rwac, vrng);
    while now.elapsed().unwrap().as_millis() < 50 {
        let mut u64txn = u64gen.get();
        loop {
            let chk = u64txn.clone();
            u64txn = match u64txn.go() {
                RWClosure::Rd(txn, _) => {
                    assert!(&chk == &txn);
                    txn.rd(U64Map(Some((0, Some(U64Tup(0, 0))))))
                }
                RWClosure::Wr(txn, _) => {
                    assert!(&chk == &txn);
                    txn.wr()
                }
                RWClosure::Cl(txn, _) => {
                    assert!(&chk == &txn);
                    break;
                }
                RWClosure::Op(txn) => {
                    txn.op()
                }
            }
        }
    }
}

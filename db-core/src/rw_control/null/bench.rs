const NULL_WRITE: bool = false;

#[test]
fn run_u64_unif() {
    // dependencies
    use db_test::core_workload::int::unif::*;
    use crate::rw_durable::null::*;
    use crate::tx_service::m_thread::*;
    println!();
    // parameter, if set to non-zero, testing will be super slow!
    for n in 1..=8 {
        println!("===");
        println!("{n} threads");
        // durablility control, null control
        let dur = Null::<U64Tup, U64Txn>::new(0, 0, NULL_WRITE);
        // concurrency control, kv sparkle in this module
        let con = super::Null::<U64Tup, U64Txn>::new();
        // service, multi-thread service
        let srv = MThreadService::new(n, |x| x, con, dur);
        // run service
        widget::u64_little_bench(srv);
    }
}

#[test]
fn run_revm_10key() {
    // dependencies
    use db_test::core_workload::eth::revm_interp::*;
    use crate::rw_durable::null::*;
    use crate::tx_service::m_thread::*;
    println!();
    for n in 1..=8 {
        println!("{n} threads");
        // durability control, null control
        let dur = Null::<EVMU256Tup, REVMInterpTxn>::new(0, 0, NULL_WRITE);
        // concurrency control, kv sparkle in this module
        let con = super::Null::<EVMU256Tup, REVMInterpTxn>::new();
        // service, multi-thread service
        let srv = MThreadService::new(n, |x| x, con, dur);
        widget::revm_10k_bench(srv);
        println!("===");
    }
}
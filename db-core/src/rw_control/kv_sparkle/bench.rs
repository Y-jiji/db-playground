const NULL_WRITE: bool = false;

#[test]
fn run_u64_unif() {
    println!("{}:{}", file!(), line!());
    // dependencies
    use db_test::core_workload::int::unif::*;
    use crate::rw_durable::null::*;
    use crate::tx_service::m_thread::*;
    use super::*;
    // durablility control, null control
    let dur = Null::<U64Tup, KVSparkleTx<U64Tup, U64Txn>>::new(5, 5, NULL_WRITE);
    // concurrency control, kv sparkle in this module
    let con = super::KVSparkle::<U64Txn, U64Tup>::new();
    // service, multi-thread service
    let srv = MThreadService::new(4, KVSparkleTx::new, con, dur);
    widget::u64_little_bench(srv);
}

#[test]
fn run_revm_10key() {
    println!("{}:{}", file!(), line!());
    // dependencies
    use db_test::core_workload::eth::revm_interp::*;
    use crate::rw_durable::null::*;
    use crate::tx_service::m_thread::*;
    use super::*;
    // durability control, null control
    let dur = Null::<EVMU256Tup, KVSparkleTx<EVMU256Tup, REVMInterpTxn>>::new(5, 5, NULL_WRITE);
    // concurrency control, kv sparkle in this module
    let con = super::KVSparkle::<REVMInterpTxn, EVMU256Tup>::new();
    // service, multi-thread service
    let srv = MThreadService::new(4, KVSparkleTx::new, con, dur);
    widget::revm_10k_bench(srv);
}
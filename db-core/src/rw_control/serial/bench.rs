const NULL_WRITE: bool = false;

#[test]
fn run_u64_unif() {
    // dependencies
    use crate::rw_durable::null::*;
    use crate::tx_service::m_thread::*;
    use db_test::core_workload::int::unif::*;
    // durablility control, null control
    let dur = Null::<U64Tup, U64Txn>::new(0, 0, NULL_WRITE);
    // concurrency control, kv sparkle in this module
    let con = super::Serial::<U64Txn, U64Tup>::new();
    // service, multi-thread service
    let srv = MThreadService::new(8, |x| x, con, dur);
    // run service
    widget::u64_little_bench(srv);
}

#[test]
fn run_revm_10key() {
    // dependencies
    use db_test::core_workload::eth::revm_interp::*;
    use crate::rw_durable::null::*;
    use crate::tx_service::m_thread::*;
    // durability control, null control
    let dur = Null::<EVMU256Tup, REVMInterpTxn>::new(0, 0, NULL_WRITE);
    // concurrency control, kv sparkle in this module
    let con = super::Serial::<REVMInterpTxn, EVMU256Tup>::new();
    // service, multi-thread service
    let srv = MThreadService::new(8, |x| x, con, dur);
    widget::revm_10k_bench(srv);
}
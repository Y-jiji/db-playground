const NULL_WRITE: bool  = false;
const RD_LATENCY: u64   = 10;
const WR_LATENCY: u64   = 10;
const NR_WORKERS: usize = 4;

#[test]
fn run_u64_unif() {
    // find this test easily
    println!("{}:{}", file!(), line!());
    // dependencies
    use db_test::core_workload::int::unif::*;
    use crate::rw_durable::null::*;
    use crate::tx_service::m_thread::*;
    use super::*;
    // durablility control, null control
    let dur = Null::<U64Tup, KVSpliceTx<U64Tup, U64Txn>>::new(RD_LATENCY, WR_LATENCY, NULL_WRITE);
    // concurrency control, kv sparkle in this module
    let con = super::KVSplice::<U64Txn, U64Tup>::new();
    // service, multi-thread service
    let srv = MThreadService::new(NR_WORKERS, KVSpliceTx::new, con, dur);
    preset::u64_little_bench(srv);
}

#[test]
fn run_revm_10key() {
    // find this test easily
    println!("{}:{}", file!(), line!());
    // dependencies
    use db_test::core_workload::eth::revm_interp::*;
    use crate::rw_durable::null::*;
    use crate::tx_service::m_thread::*;
    use super::*;
    // durability control, null control
    let dur = Null::<EVMU256Tup, KVSpliceTx<EVMU256Tup, REVMInterpTxn>>::new(RD_LATENCY, WR_LATENCY, NULL_WRITE);
    // concurrency control, kv sparkle in this module
    let con = super::KVSplice::<REVMInterpTxn, EVMU256Tup>::new();
    // service, multi-thread service
    let srv = MThreadService::new(NR_WORKERS, KVSpliceTx::new, con, dur);
    preset::revm_10k_bench(srv);
}
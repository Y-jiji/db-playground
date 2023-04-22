use typing::tx::*;
use super::*;

pub fn u64_little_bench(mut service: impl TxService<U64Txn, U64Tup>) -> Vec<Option<u64>>
{
    use std::time::*;
    const N_TXN: u64 = 25000;
    const N_WARMUP: u64 = 10000;
    const RWAC: (u64, u64, u64, u64) = (15, 15, 1, 4);
    const VRNG: u64 = 20000;
    const SEED: u64 = 1145141919810;
    let mut workload = U64Gen::new(SEED, RWAC, VRNG);
    println!("start service");
    service.start().unwrap_or_else(|_| panic!("fail to start service"));
    print!("warm up [0/{N_WARMUP}]          \r");
    for _i in 0..N_WARMUP {
        let txn = workload.get();
        if txn.id() == 0 { continue; }
        service.put(txn).unwrap_or_else(|_| panic!("fail to put transaction {_i}"));
        print!("warm up [{}/{N_WARMUP}]         \r", _i+1);
    }
    println!();
    let start_time = SystemTime::now();
    print!("put [0/{N_TXN}]         \r");
    for _i in N_WARMUP..(N_WARMUP+N_TXN) {
        let txn = workload.get();
        if txn.id() == 0 { continue; }
        service.put(txn).unwrap_or_else(|_| panic!("fail to put transaction {_i}"));
        print!("put [{}/{N_TXN}]        \r", _i+1-N_WARMUP);
    }
    println!();
    print!("get [0/{N_TXN}]         \r");
    let mut output = vec![];
    for _i in N_WARMUP..(N_WARMUP+N_TXN) {
        let wait = 10;
        loop {
            if let Ok(x) = service.get(_i) {
                output.push(x);
                break;
            } else {
                std::thread::sleep(Duration::from_nanos(wait));
            }
        }
        print!("get [{}/{N_TXN}]        \r", _i+1-N_WARMUP);
    }
    println!();
    println!("elapsed {:.4} (sec)", start_time.elapsed().unwrap().as_secs_f32());
    println!("throughput {:.4} (txn/sec)", N_TXN as f64 / start_time.elapsed().unwrap().as_secs_f64());
    #[cfg(feature="internal_info")]
    for i in 0..N_TXN {
        println!("output {i:<8}:{:?}", output[i as usize]);
    }
    println!("close service");
    if let Err(_) = service.close() {
        println!("service stop with error");
    }
    return output;
}

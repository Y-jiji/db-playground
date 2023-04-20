use super::*;
use rand::random;
use typing::tx::*;

#[test]
fn stepping() {
    let seed = random::<u64>();
    let rwac = (1, 1, 1, 1);
    let vrng = 1000u64;
    let mut u64gen = U64Gen::new(seed, rwac, vrng);
    let mut line = String::new();
    loop {
        let mut u64txn = u64gen.get();
        let mut opscnt = 0;
        eprintln!("====================");
        loop {
            opscnt += 1;
            u64txn = match u64txn.go() {
                RWClosure::Rd(txn, U64Prp(qry)) => {
                    eprintln!("{:<5} read  {:>10}", txn.id(), qry);
                    txn.rd(U64Map(Some((0, None))))
                }
                RWClosure::Wr(txn, U64Map(Some(map))) => {
                    eprintln!("{:<5} write {:>10} <- {:?}", txn.id(), map.0, map.1);
                    txn.wr()
                }
                RWClosure::Wr(txn, U64Map(None)) => {
                    eprintln!("{:<5} write nothing", txn.id());
                    txn.wr()
                }
                RWClosure::Cl(txn, ending) => {
                    match ending {
                        End::Abort => eprintln!("{:<5} abort", txn.id()),
                        End::Ready => {
                            eprintln!("{:<5} ready (output: {})", txn.id(), txn.cl().unwrap())
                        }
                    }
                    break;
                }
                _ => unreachable!(),
            }
        }
        eprintln!("operations count: {opscnt}");
        eprintln!("====================");
        eprintln!(
            "press enter to see the next transaction, send EOF (Ctrl+Z on windows) to terminate"
        );
        let x = std::io::stdin().read_line(&mut line).unwrap();
        if x == 0 {
            break;
        }
    }
}

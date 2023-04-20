use auto_impl::auto_impl;

// (CoreW)ork(l)oad interface
#[auto_impl(Box)]
pub trait CoreWL {
    type Txn;
    fn get(&mut self) -> Self::Txn;
}

macro_rules! CoreWLDerive {
    ($WLType: ty, $TxType: ty) => {
        impl CoreWL for $WLType {
            type Txn = $TxType;
            fn get(&mut self) -> Self::Txn {
                self.get()
            }
        }
    };
}

// interger transaction
pub mod int;
CoreWLDerive!(int::unif::U64Gen, int::unif::U64Txn);

// ethereum benchmarks
pub mod eth;
CoreWLDerive!(eth::revm_interp::REVMInterpGen, eth::revm_interp::REVMInterpTxn);

// tpcc benchmark
pub mod tpcc;
// ## PEVM interpreter version
//! EVM with persistent memory to make fine-grained rollbacks. 
//! 

mod txn;
pub use txn::*;

pub struct PEVMInterp {
    
}
//! ## Key-Value Sparkle (Single Sharded)
//! 
//! > Li, Zhongmiao, Paolo Romano, and Peter Van Roy. "Sparkle: speculative deterministic concurrency control for partially replicated transactional stores." 2019 49th Annual IEEE/IFIP International Conference on Dependable Systems and Networks (DSN). IEEE, 2019.
//! 
//! In this module we implement a single sharded version of sparkle, a determined concurrency control protocol. 
//! For simplicity, we only work with a key-value queries. 
//! We may implement a multi-sharded version in the future. 

// a table
mod table;
// transaction pool (a small widget to store suspended transactions)
mod tpool;
// a simple wrapper adding auxilary information to a common transaction
mod twrap;

// sparkle error
mod error;
// core sparkle protocol implementation
mod proto;

pub use twrap::*;
pub use proto::*;
pub use error::*;

#[cfg(test)]
mod bench; // inlined benchmark module to detect performance issues
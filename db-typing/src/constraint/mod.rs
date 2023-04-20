// checkpointing on transaction
mod ckpt;
pub use ckpt::*;

// natural number
mod nat;
pub use nat::*;

// proposition
mod filter; // the most loose contraints on set proposition
pub use filter::*;
mod indexer; // directly index an element
pub use indexer::*;

// mapping
mod mapper; // 
pub use mapper::*;

// abstraction over a lock manager
mod lap;
pub use lap::*;

// identity on a data item
mod identity;
pub use identity::*;

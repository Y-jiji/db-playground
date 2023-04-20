// --------------------- common traits --------------------- //

// read-write control interface traits
use typing::rw as rw;

// transaction interface and transaction service
use typing::tx as tx;

// ------------------- common utilities ------------------- //

// internal utilities
pub mod utilities;

// trait constraints on transactions
use typing::constraint as constraint;

// ----------------- core implementations ---------------- //

// read/write concurrency control implementations
pub mod rw_control;

// read/write durability control implementations
pub mod rw_durable;

// transaction service implementations
pub mod tx_service;

// hardware abstractions
pub mod hardware;
mod proto;
pub use proto::*;

#[cfg(test)]
mod bench; // inlined benchmark module to detect performance issues
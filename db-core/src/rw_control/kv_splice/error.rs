#[derive(Debug)]
pub enum KVSpliceErr<DErr> {
    External(DErr),  
}
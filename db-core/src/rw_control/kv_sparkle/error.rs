#[derive(Debug)]
pub enum KVSparkleErr<DErr> {
    External(DErr),  
}
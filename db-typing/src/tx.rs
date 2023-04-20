pub trait Tx<V>
where
    Self: Sized,
{
    /// a transaction id that will uniquely identify a transaction from others
    /// transaction id can be adjusted with other traits
    type I;
    fn id(&self) -> Self::I;

    /// one of the key message passing types, 
    /// conceptually represents a proposition on data entries
    type Prp;

    /// one of the key message passing types, 
    /// conceptually represents a map from transaction id to data entry 
    type Map;

    /// one of the key message passing types, 
    /// output of a transaction, may contain some extra logic even after a transaction is done
    type Out;

    /// release a closure that will either stuck a transaction with external r/w requests
    /// or just release a forwarding internal transition
    fn go(self) -> RWClosure<Self, Self::Prp, Self::Map>;

    /// proceed a read-stuck transaction by plugging in map information
    fn rd(self, map: Self::Map) -> Self;

    /// proceed a write-stuck transaction by plugging in ... nothing
    fn wr(self) -> Self;

    /// perform an internal operation step
    fn op(self) -> Self;

    /// close a transaction and release output
    fn cl(self) -> Option<Self::Out>;
}

#[derive(Debug, Clone, Copy)]
pub enum End {
    Ready, // ready to commit
    Abort, // ready to abort
}

/// a state that will stick to wait external event or information
#[derive(Debug)]
#[repr(u8)]
pub enum RWClosure<T, Prp, Map> {
    Op(T) = 0x0,      // perform some internal computation
    Rd(T, Prp) = 0x1, // stick to read external data
    Wr(T, Map) = 0x2, // stick to write external data
    Cl(T, End) = 0x3, // close current transaction
}

/// transaction processing service is a running database instance or just a client
pub trait TxService<T: Tx<V>, V> {
    /// specific error type of this transaction
    type Err;

    /// start a transaction processing unit
    fn start(&mut self) -> Result<(), Self::Err>;

    /// close this service and get an error report if there is any
    fn close(&mut self) -> Result<(), Self::Err>;

    /// put (send) a transaction to a running database instance
    fn put(&self, t: T) -> Result<(), Self::Err>;

    /// get outputs of a given transaction, if this transaction is not done properly, an error will be released in the form of Self::Err
    fn get(&self, i: T::I) -> Result<Option<T::Out>, Self::Err>;
}
pub trait Id {
    type I;
    fn id(&self) -> Self::I;
}
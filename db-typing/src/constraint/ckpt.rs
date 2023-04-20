pub trait TxCkpt
where
    Self::Ckpt: Sized + Clone + Copy,
{
    type Ckpt;
    fn make(&mut self) -> Self::Ckpt;
    fn goto(&mut self, ckpt: Self::Ckpt);
}

pub type Ckpt<T> = <T as TxCkpt>::Ckpt;
pub trait Nat {
    fn zero() -> Self;
    fn succ(&self) -> Self;
}

macro_rules! these_are_nat {
    ($($T: ty)*) => {
        $(
        impl Nat for $T {
            fn zero() -> Self { 0 as $T }
            fn succ(&self) -> Self { *self + 1 as $T }
        })*
    };
}

these_are_nat! {
    u128 usize u64 u32 u16
    i128 isize i64 i32 i16
    f32 f64
}
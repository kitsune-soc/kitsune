mod v0_2;
mod v1;

pub trait Compat {
    type Output;

    fn compat(self) -> Self::Output;
}

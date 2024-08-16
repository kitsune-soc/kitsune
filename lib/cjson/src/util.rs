use sonic_rs::writer::WriteExt;
use std::io;

macro_rules! for_both {
    ($owner:ident, $matcher:pat => $impl:expr) => {{
        match $owner {
            Self::Left($matcher) => $impl,
            Self::Right($matcher) => $impl,
        }
    }};
}

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> io::Write for Either<L, R>
where
    L: io::Write,
    R: io::Write,
{
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for_both!(self, inner => inner.write(buf))
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        for_both!(self, inner => inner.write_all(buf))
    }

    #[inline]
    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> io::Result<()> {
        for_both!(self, inner => inner.write_fmt(fmt))
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        for_both!(self, inner => inner.flush())
    }
}

impl<L, R> WriteExt for Either<L, R>
where
    L: WriteExt + io::Write,
    R: WriteExt + io::Write,
{
    #[inline]
    fn reserve_with(
        &mut self,
        additional: usize,
    ) -> std::io::Result<&mut [std::mem::MaybeUninit<u8>]> {
        for_both!(self, inner => inner.reserve_with(additional))
    }

    #[inline]
    #[allow(unsafe_code)] // We just dispatch over already unsafe implementations
    unsafe fn flush_len(&mut self, additional: usize) -> std::io::Result<()> {
        for_both!(self, inner => inner.flush_len(additional))
    }
}

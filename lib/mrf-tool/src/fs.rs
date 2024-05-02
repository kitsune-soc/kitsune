use std::{
    collections::HashMap,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

pub trait Filesystem {
    type File<'a>: io::Write
    where
        Self: 'a;

    fn copy(&mut self, src: &Path, dst: &Path) -> io::Result<()>;
    fn read(&mut self, path: &Path) -> io::Result<Vec<u8>>;

    fn create_or_truncate(&mut self, path: &Path) -> io::Result<Self::File<'_>>;
    fn open_append(&mut self, path: &Path) -> io::Result<Self::File<'_>>;
}

pub struct DummyFs {
    inner: HashMap<PathBuf, Vec<u8>>,
}

#[inline]
fn file_not_found() -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, "file not found")
}

impl Filesystem for DummyFs {
    type File<'a> = &'a mut Vec<u8>;

    #[inline]
    fn copy(&mut self, src: &Path, dst: &Path) -> io::Result<()> {
        let value = self.inner.get(src).ok_or_else(file_not_found)?;
        self.inner.insert(dst.to_path_buf(), value.clone());

        Ok(())
    }

    #[inline]
    fn read(&mut self, path: &Path) -> io::Result<Vec<u8>> {
        self.inner.get(path).cloned().ok_or_else(file_not_found)
    }

    #[inline]
    fn create_or_truncate(&mut self, path: &Path) -> io::Result<Self::File<'_>> {
        Ok(self.inner.entry(path.to_path_buf()).or_default())
    }

    #[inline]
    fn open_append(&mut self, path: &Path) -> io::Result<Self::File<'_>> {
        self.inner.get_mut(path).ok_or_else(file_not_found)
    }
}

#[derive(Default)]
pub struct NativeFs {
    _priv: (),
}

impl Filesystem for NativeFs {
    type File<'a> = File;

    #[inline]
    fn copy(&mut self, src: &Path, dst: &Path) -> io::Result<()> {
        fs::copy(src, dst)?;
        Ok(())
    }

    #[inline]
    fn read(&mut self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    #[inline]
    fn create_or_truncate(&mut self, path: &Path) -> io::Result<Self::File<'_>> {
        File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
    }

    #[inline]
    fn open_append(&mut self, path: &Path) -> io::Result<Self::File<'_>> {
        File::options().append(true).open(path)
    }
}

use color_eyre::eyre::{self, Result};
use std::path::Path;
use wasmtime::component::Component;

pub struct Cache {
    inner: sled::Db,
}

impl Cache {
    #[inline]
    pub fn open<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            inner: sled::open(path)?,
        })
    }

    #[inline]
    #[instrument(skip_all)]
    pub fn load(&self, engine: &wasmtime::Engine, component: &[u8]) -> Result<Option<Component>> {
        let hash = blake3::hash(component);
        let Some(precompiled) = self.inner.get(hash.as_bytes())? else {
            return Ok(None);
        };

        debug!(hash = %hash.to_hex(), "hit component cache");

        // SAFETY: The function is defined as unsafe since it is only doing very simple checks whether the precompiled component inside is actually valid
        //         But since we source our cache from disk, we can assume that the files are fine. If they aren't, the user has tempered with them or they were otherwise corrupted.
        //         If that's the case the user has bigger issues than a little memory unsafety here. And it's also nothing we can really protect against.
        #[allow(unsafe_code)]
        Ok(unsafe { Component::deserialize(engine, precompiled).ok() })
    }

    #[inline]
    #[instrument(skip_all)]
    pub fn store(&self, source: &[u8], component: &Component) -> Result<()> {
        let hash = blake3::hash(source);
        self.inner.insert(
            hash.as_bytes(),
            component.serialize().map_err(eyre::Report::msg)?,
        )?;

        debug!(hash = %hash.to_hex(), "stored component in cache");

        Ok(())
    }
}

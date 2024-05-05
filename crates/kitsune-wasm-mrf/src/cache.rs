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
    pub fn load(
        &self,
        engine: &wasmtime::Engine,
        component_src: &[u8],
    ) -> Result<Option<Component>> {
        let hash = blake3::hash(component_src);
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
    pub fn store(&self, component_src: &[u8], component: &Component) -> Result<()> {
        let hash = blake3::hash(component_src);
        self.inner.insert(
            hash.as_bytes(),
            component.serialize().map_err(eyre::Report::msg)?,
        )?;

        debug!(hash = %hash.to_hex(), "stored component in cache");

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Cache;
    use wasmtime::component::Component;

    #[test]
    fn roundtrip() {
        let tempdir = tempfile::tempdir().unwrap();
        let cache = Cache::open(tempdir.path()).unwrap();

        let mut config = wasmtime::Config::new();
        config.wasm_component_model(true);
        let engine = wasmtime::Engine::new(&config).unwrap();

        let component_src = wat::parse_str("( component )").unwrap();
        let component = Component::from_binary(&engine, &component_src).unwrap();

        cache.store(&component_src, &component).unwrap();
        let loaded_component = cache.load(&engine, &component_src).unwrap().unwrap();

        assert_eq!(
            loaded_component.serialize().unwrap(),
            component.serialize().unwrap()
        );
    }
}

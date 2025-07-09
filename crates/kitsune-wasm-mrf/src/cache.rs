use color_eyre::eyre::{self, Result};
use std::path::Path;
use wasmtime::component::Component;

const CACHE_TABLE: redb::TableDefinition<'_, [u8; blake3::OUT_LEN], &[u8]> =
    redb::TableDefinition::new("wasm_cache");

pub struct Cache {
    inner: redb::Database,
}

impl Cache {
    #[inline]
    pub fn open<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let database = redb::Database::create(path)?;
        let transaction = database.begin_write()?;
        transaction.open_table(CACHE_TABLE)?;
        transaction.commit()?;

        Ok(Self { inner: database })
    }

    #[inline]
    #[cfg_attr(not(coverage), instrument(skip_all))]
    pub fn load(
        &self,
        engine: &wasmtime::Engine,
        component_src: &[u8],
    ) -> Result<Option<Component>> {
        let transaction = self.inner.begin_read()?;
        let table = transaction.open_table(CACHE_TABLE)?;

        let hash = blake3::hash(component_src);
        let hash_hex = hash.to_hex();
        let Some(precompiled) = table.get(hash.as_bytes())? else {
            return Ok(None);
        };

        debug!(hash = %hash_hex, "hit component cache");

        // SAFETY: The function is defined as unsafe since it is only doing very simple checks whether the precompiled component inside is actually valid
        //         But since we source our cache from disk, we can assume that the files are fine. If they aren't, the user has tempered with them or they were otherwise corrupted.
        //         If that's the case the user has bigger issues than a little memory unsafety here. And it's also nothing we can really protect against.
        #[allow(unsafe_code)]
        Ok(unsafe {
            Component::deserialize(engine, precompiled.value())
                .inspect_err(|error| {
                    debug!(
                        hash = %hash_hex,
                        %error,
                        "failed to deserialize component. pretending that we don't have a component in cache",
                    );
                })
                .ok()
        })
    }

    #[inline]
    #[cfg_attr(not(coverage), instrument(skip_all))]
    pub fn store(&self, component_src: &[u8], component: &Component) -> Result<()> {
        let hash = blake3::hash(component_src);
        let serialized_component = component.serialize().map_err(eyre::Report::msg)?;

        let transaction = self.inner.begin_write()?;
        {
            let mut table = transaction.open_table(CACHE_TABLE)?;
            table.insert(hash.as_bytes(), serialized_component.as_slice())?;
        }
        transaction.commit()?;

        debug!(hash = %hash.to_hex(), "stored component in cache");

        Ok(())
    }
}

impl Drop for Cache {
    fn drop(&mut self) {
        self.inner.compact().expect("Failed to compact database");
    }
}

#[cfg(test)]
mod test {
    use super::Cache;
    use tempfile::NamedTempFile;
    use wasmtime::component::Component;

    #[test]
    fn roundtrip() {
        let db_file = NamedTempFile::new().unwrap();
        let cache = Cache::open(db_file.path()).unwrap();

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

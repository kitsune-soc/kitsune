use super::{Backend, BucketBackend};
use color_eyre::eyre;
use std::path::Path;
use triomphe::Arc;

type TableDefinition<'a> = redb::TableDefinition<'a, &'static str, &'static [u8]>;

pub struct FsBackend {
    inner: Arc<redb::Database>,
}

impl FsBackend {
    pub fn from_path<P>(path: P) -> eyre::Result<Self>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            inner: Arc::new(redb::Database::create(path)?),
        })
    }
}

impl Backend for FsBackend {
    type Bucket = FsBucketBackend;

    async fn open(&self, module_name: &str, name: &str) -> eyre::Result<Self::Bucket> {
        let table_name = format!("{module_name}:{name}");
        let transaction = self.inner.begin_write()?;
        transaction.open_table(TableDefinition::new(&table_name))?;
        transaction.commit()?;

        Ok(FsBucketBackend {
            inner: self.inner.clone(),
            table_name,
        })
    }
}

pub struct FsBucketBackend {
    inner: Arc<redb::Database>,
    table_name: String,
}

impl BucketBackend for FsBucketBackend {
    async fn exists(&self, key: &str) -> eyre::Result<bool> {
        let transaction = self.inner.begin_read()?;
        let table = transaction.open_table(TableDefinition::new(&self.table_name))?;

        match table.get(key) {
            Ok(val) => Ok(val.is_some()),
            Err(err) => Err(err.into()),
        }
    }

    async fn delete(&self, key: &str) -> eyre::Result<()> {
        let transaction = self.inner.begin_write()?;
        {
            let mut table = transaction.open_table(TableDefinition::new(&self.table_name))?;
            table.remove(key)?;
        }
        transaction.commit()?;

        Ok(())
    }

    async fn get(&self, key: &str) -> eyre::Result<Option<Vec<u8>>> {
        let transaction = self.inner.begin_read()?;
        let table = transaction.open_table(TableDefinition::new(&self.table_name))?;

        table
            .get(key)
            .map(|maybe_val| maybe_val.map(|val| val.value().to_vec()))
            .map_err(Into::into)
    }

    async fn set(&self, key: &str, value: &[u8]) -> eyre::Result<()> {
        let transaction = self.inner.begin_write()?;
        {
            let mut table = transaction.open_table(TableDefinition::new(&self.table_name))?;
            table.insert(key, value)?;
        }
        transaction.commit()?;

        Ok(())
    }
}

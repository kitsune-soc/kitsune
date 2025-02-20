//!
//! File system backed implementation of the [`StorageBackend`] trait
//!

use crate::StorageBackend;
use bytes::Bytes;
use futures_util::{Stream, TryStreamExt};
use kitsune_error::Result;
use std::{path::PathBuf, pin::pin};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tokio_util::io::ReaderStream;

#[derive(Clone)]
/// File system storage
pub struct Storage {
    storage_dir: PathBuf,
}

impl Storage {
    /// Create a new file system storage
    ///
    /// It always requires a directory the operations on it are relative to
    #[must_use]
    pub fn new(storage_dir: PathBuf) -> Self {
        Self { storage_dir }
    }
}

impl StorageBackend for Storage {
    async fn delete(&self, path: &str) -> Result<()> {
        fs::remove_file(self.storage_dir.join(path)).await?;
        Ok(())
    }

    async fn get(&self, path: &str) -> Result<impl Stream<Item = Result<Bytes>> + 'static> {
        let file = File::open(self.storage_dir.join(path)).await?;
        Ok(ReaderStream::new(file).map_err(Into::into))
    }

    async fn put<T>(&self, path: &str, input_stream: T) -> Result<()>
    where
        T: Stream<Item = Result<Bytes>> + Send + Sync + 'static,
    {
        let mut input_stream = pin!(input_stream);
        let mut file = File::create(self.storage_dir.join(path)).await?;
        while let Some(chunk) = input_stream.try_next().await? {
            file.write_all(&chunk).await?;
        }
        file.flush().await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{StorageBackend, fs::Storage};
    use bytes::{BufMut, BytesMut};
    use futures_util::{TryStreamExt, future, stream};
    use std::str;
    use tempfile::TempDir;

    const TEST_TEXT: &str = r"
    新時代はこの未来だ
    世界中全部 変えてしまえば 変えてしまえば
    ジャマモノ やなもの なんて消して
    この世とメタモルフォーゼしようぜ
    ミュージック キミが起こす マジック
    目を閉じれば未来が開いて
    いつまでも終わりが来ないようにって
    この歌を歌うよ
    Do you wanna play? リアルゲーム ギリギリ
    綱渡りみたいな旋律 認めない戻れない忘れたい
    夢の中に居させて I wanna be free
    見えるよ新時代が 世界の向こうへ
    さあ行くよ new world
    新時代はこの未来だ
    世界中全部 変えてしまえば 変えてしまえば
    果てしない音楽がもっと届くように
    夢は見ないわ キミが話した「ボクを信じて」
    Ooh
    あれこれいらないものは消して
    リアルをカラフルに越えようぜ
    ミュージック 今始まる ライジング
    目をつぶりみんなで逃げようよ
    今よりイイモノを見せてあげるよ
    この歌を歌えば
    Do you wanna play? リアルゲーム ギリギリ
    綱渡りみたいな運命 認めない戻れない忘れたい
    夢の中に居させて I wanna be free
    見えるよ新時代が 世界の向こうへ
    さあ行くよ new world
    信じたいわ この未来を
    世界中全部 変えてしまえば 変えてしまえば
    果てしない音楽がもっと届くように
    夢を見せるよ 夢を見せるよ 新時代だ
    Ooh
    新時代だ";

    #[tokio::test]
    async fn basic() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path().to_owned());

        storage
            .put("hello-world", stream::once(future::ok(TEST_TEXT.into())))
            .await
            .unwrap();

        let data_stream = storage.get("hello-world").await.unwrap();
        let data = data_stream
            .try_fold(BytesMut::new(), |mut acc, chunk| {
                acc.put(chunk);
                future::ok(acc)
            })
            .await
            .unwrap();
        let data = str::from_utf8(&data).unwrap();

        assert_eq!(TEST_TEXT, data);

        storage.delete("hello-world").await.unwrap();
        assert!(storage.get("hello-world").await.is_err());
    }
}

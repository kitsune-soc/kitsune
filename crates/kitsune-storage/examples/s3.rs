use bytes::{BufMut, BytesMut};
use futures_util::{future, stream, TryStreamExt};
use kitsune_storage::{s3::Storage, StorageBackend};
use rusty_s3::{Bucket, Credentials, UrlStyle};
use std::{env, str};

const TEST_DATA: &str = r#"
散々な思い出は悲しみを穿つほど
やるせない恨みはアイツのために
置いてきたのさ
あんたらわかっちゃないだろ
本当に傷む孤独を
今だけ箍外してきて
怒りよ今 悪党ぶっ飛ばして
そりゃあ愛ある罰だ
もう眠くはないや ないやないや
もう悲しくないさ ないさ
そう 怒りよ今 悪党蹴り飛ばして
そりゃあ愛への罰だ
もう眠くはないな ないなないな
もう寂しくないさ ないさ
逆光よ (na-na-na, na-na-na-na-na)
(Na-na-na, na-na-na-na-na)
(Na-na-na, na-na-na-na-na, na-na-na-na-na-na)
惨憺たる結末は美しさを纏うほど
限りなく 体温に近い「赤」に彩られていた
散漫な視界でも美しさがわかるほど
焼き付ける光を背に受ける
「赤」に気を取られている
もつれてしまった心は 解っている今でも
ほつれてしまった言葉が焦っている
怒りよ今 悪党ぶっ飛ばして
そりゃあ愛ある罰だ
もう眠くはないや ないやないや
もう悲しくないさ ないさ
そう 怒りよ今 悪党蹴り飛ばして
そりゃあ愛への罰だ
もう眠くはないな ないなないな
もう寂しくないさ ないさ
逆光よ
もう 怒り願った言葉は
崩れ へたってしまったが
今でも未練たらしくしている
あぁ 何度も放った言葉が
届き 解っているのなら
なんて 夢見が苦しいから
もう怒りよ また 悪党ぶっ飛ばして
そりゃあ愛ある罰だ
もう眠くはないや ないやないや
もう悲しくないさ ないさ
そう 怒りよさぁ 悪党ふっ飛ばして
そりゃあ愛への罰だ
もう眠くはないな ないなないな
もう寂しくないさ ないさ
逆光よ (na-na-na, na-na-na-na-na)
(Na-na-na, na-na-na-na-na)
(Na-na-na, na-na-na-na-na, na-na-na-na-na-na)"#;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let bucket_name = env::var("BUCKET_NAME").unwrap();
    let access_key = env::var("ACCESS_KEY").unwrap();
    let secret_access_key = env::var("SECRET_ACCESS_KEY").unwrap();
    let endpoint_url = env::var("ENDPOINT_URL").unwrap();
    let region = env::var("REGION").unwrap();

    let credentials = Credentials::new(access_key, secret_access_key);
    let bucket = Bucket::new(
        endpoint_url.parse().unwrap(),
        UrlStyle::VirtualHost,
        bucket_name,
        region,
    )
    .unwrap();
    let backend = Storage::new(bucket, credentials);

    let operation = env::args().nth(1).unwrap();

    match operation.as_str() {
        "delete" => backend.delete("very-important.txt").await.unwrap(),
        "get" => {
            let raw_data = backend
                .get("very-important.txt")
                .await
                .unwrap()
                .try_fold(BytesMut::new(), |mut acc, chunk| {
                    acc.put(chunk);
                    future::ok(acc)
                })
                .await
                .unwrap();

            println!("{}", str::from_utf8(&raw_data).unwrap());
        }
        "put" => backend
            .put(
                "very-important.txt",
                stream::once(future::ok(TEST_DATA.into())),
            )
            .await
            .unwrap(),
        _ => eprintln!("unknown operation"),
    }
}

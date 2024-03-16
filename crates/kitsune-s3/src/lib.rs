use bytes::Bytes;
use futures_util::{Stream, TryStreamExt};
use http::{header::CONTENT_LENGTH, Request};
use kitsune_http_client::{Body, Client as HttpClient, Response};
use rusty_s3::{actions::CreateMultipartUpload, Bucket, Credentials, S3Action};
use serde::Serialize;
use std::time::Duration;
use typed_builder::TypedBuilder;

type BoxError = Box<dyn std::error::Error + Send + Sync>;
type Result<T, E = BoxError> = std::result::Result<T, E>;

const TWO_MINUTES: Duration = Duration::from_secs(2 * 60);

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct CreateBucketConfiguration<'a> {
    location_constraint: &'a str,
}

#[inline]
const fn s3_method_to_http(method: rusty_s3::Method) -> http::Method {
    match method {
        rusty_s3::Method::Head => http::Method::HEAD,
        rusty_s3::Method::Get => http::Method::GET,
        rusty_s3::Method::Post => http::Method::POST,
        rusty_s3::Method::Put => http::Method::PUT,
        rusty_s3::Method::Delete => http::Method::DELETE,
    }
}

#[inline]
const fn http_method_by_value<'a, T: ?Sized>(_: &T) -> http::Method
where
    T: S3Action<'a>,
{
    s3_method_to_http(T::METHOD)
}

async fn execute_request(client: &HttpClient, req: Request<Body>) -> Result<Response> {
    let response = client.execute(req).await?;
    if !response.status().is_success() {
        let mut err_msg = format!("s3 request failed: {response:?}");

        let body = response.text().await?;
        err_msg.push_str("\nbody: ");
        err_msg.push_str(&body);

        return Err(Box::from(err_msg));
    }

    Ok(response)
}

#[derive(TypedBuilder)]
pub struct Client {
    bucket: Bucket,
    credentials: Credentials,
    #[builder(
        default = HttpClient::builder().content_length_limit(None).build(),
        setter(skip),
    )]
    http_client: HttpClient,
}

// Note: We use `String::from(url::Url)` here since this uses a specialized implementation which avoids reallocating
// Since the `Url` type already contains a serialized `String` version of itself.
// Its `From<Url> for String` impl just returns ownership of this internal string instead of allocating a new buffer and copying the data.

impl Client {
    pub async fn create_bucket(&self) -> Result<()> {
        let create_action = self.bucket.create_bucket(&self.credentials);
        let body = quick_xml::se::to_string(&CreateBucketConfiguration {
            location_constraint: self.bucket.region(),
        })?;

        let request = Request::builder()
            .uri(String::from(create_action.sign(TWO_MINUTES)))
            .method(http_method_by_value(&create_action))
            .body(Body::data(body))?;

        execute_request(&self.http_client, request).await?;

        Ok(())
    }

    pub async fn delete_bucket(&self) -> Result<()> {
        let delete_action = self.bucket.delete_bucket(&self.credentials);

        let request = Request::builder()
            .uri(String::from(delete_action.sign(TWO_MINUTES)))
            .method(http_method_by_value(&delete_action))
            .body(Body::empty())?;

        execute_request(&self.http_client, request).await?;

        Ok(())
    }

    pub async fn delete_object(&self, path: &str) -> Result<()> {
        let delete_action = self.bucket.delete_object(Some(&self.credentials), path);

        let request = Request::builder()
            .uri(String::from(delete_action.sign(TWO_MINUTES)))
            .method(http_method_by_value(&delete_action))
            .body(Body::empty())?;

        execute_request(&self.http_client, request).await?;

        Ok(())
    }

    pub async fn get_object(&self, path: &str) -> Result<impl Stream<Item = Result<Bytes>>> {
        let get_action = self.bucket.get_object(Some(&self.credentials), path);

        let request = Request::builder()
            .uri(String::from(get_action.sign(TWO_MINUTES)))
            .method(http_method_by_value(&get_action))
            .body(Body::empty())?;

        let response = execute_request(&self.http_client, request).await?;

        Ok(response.stream().map_err(Into::into))
    }

    pub async fn put_object<S, E>(&self, path: &str, stream: S) -> Result<()>
    where
        S: Stream<Item = Result<Bytes, E>> + Send + Sync + 'static,
        E: Into<BoxError>,
    {
        let create_multipart_upload = self
            .bucket
            .create_multipart_upload(Some(&self.credentials), path);

        let request = Request::builder()
            .uri(String::from(create_multipart_upload.sign(TWO_MINUTES)))
            .method(http_method_by_value(&create_multipart_upload))
            .body(Body::empty())?;

        let response = execute_request(&self.http_client, request)
            .await?
            .text()
            .await?;
        let create_response = CreateMultipartUpload::parse_response(&response)?;

        futures_util::pin_mut!(stream);

        let mut ctr = 1;
        while let Some(chunk) = stream.try_next().await.map_err(Into::into)? {
            let upload_part = self.bucket.upload_part(
                Some(&self.credentials),
                path,
                ctr,
                create_response.upload_id(),
            );

            let request = Request::builder()
                .header(CONTENT_LENGTH, chunk.len())
                .uri(String::from(upload_part.sign(TWO_MINUTES)))
                .method(http_method_by_value(&upload_part))
                .body(Body::data(chunk))?;

            execute_request(&self.http_client, request).await?;

            ctr += 1;
        }

        let complete_multipart_upload = self.bucket.complete_multipart_upload(
            Some(&self.credentials),
            path,
            create_response.upload_id(),
            (&[] as &[&str]).iter().copied(),
        );

        let request = Request::builder()
            .uri(String::from(complete_multipart_upload.sign(TWO_MINUTES)))
            .method(http_method_by_value(&complete_multipart_upload))
            .body(Body::data(complete_multipart_upload.body()))?;

        execute_request(&self.http_client, request).await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{BoxError, CreateBucketConfiguration};
    use futures_util::{future, stream, TryStreamExt};
    use kitsune_test::minio_test;

    const TEST_DATA: &[u8] = b"https://open.spotify.com/track/6VNNakpjSH8LNBX7fSGhUv";

    #[test]
    fn create_bucket_configuration() {
        let config = CreateBucketConfiguration {
            location_constraint: "neptune",
        };
        let encoded = quick_xml::se::to_string(&config).unwrap();

        assert_eq!(
            encoded,
            "<CreateBucketConfiguration>\
                <LocationConstraint>neptune</LocationConstraint>\
            </CreateBucketConfiguration>"
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn full_test() {
        minio_test(|client| async move {
            client
                .put_object(
                    "good song",
                    stream::once(future::ok::<_, BoxError>(TEST_DATA.into())),
                )
                .await
                .unwrap();

            let data = client
                .get_object("good song")
                .await
                .unwrap()
                .try_fold(Vec::new(), |mut acc, chunk| async move {
                    acc.extend_from_slice(&chunk);
                    Ok(acc)
                })
                .await
                .unwrap();

            assert_eq!(data, TEST_DATA);

            client.delete_object("good song").await.unwrap();

            let result = client.get_object("good song").await;
            assert!(result.is_err());
        })
        .await;
    }
}

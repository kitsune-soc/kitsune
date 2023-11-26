use kitsune_config::storage;
use kitsune_storage::{fs::Storage as FsStorage, s3::Storage as S3Storage, AnyStorageBackend};

pub fn storage(config: &storage::Configuration) -> eyre::Result<AnyStorageBackend> {
    let storage = match config {
        storage::Configuration::Fs(ref fs_config) => {
            FsStorage::new(fs_config.upload_dir.as_str().into()).into()
        }
        storage::Configuration::S3(ref s3_config) => {
            let path_style = if s3_config.force_path_style {
                rusty_s3::UrlStyle::Path
            } else {
                rusty_s3::UrlStyle::VirtualHost
            };

            let s3_credentials = rusty_s3::Credentials::new(
                s3_config.access_key.as_str(),
                s3_config.secret_access_key.as_str(),
            );
            let s3_bucket = rusty_s3::Bucket::new(
                s3_config.endpoint_url.parse()?,
                path_style,
                s3_config.bucket_name.to_string(),
                s3_config.region.to_string(),
            )?;

            S3Storage::new(s3_bucket, s3_credentials).into()
        }
    };

    Ok(storage)
}

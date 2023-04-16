use crate::{util::UnixTimestampExt, Error, Result, SignatureComponent};
use base64::{engine::general_purpose, Engine};
use derive_builder::Builder;
use http::{header::DATE, request::Parts};
use std::{
    cmp::min,
    time::{Duration, SystemTime, SystemTimeError},
};
use time::{format_description::well_known::Rfc2822, OffsetDateTime};

#[derive(Builder)]
pub struct SignatureHeader<'a> {
    pub key_id: &'a str,
    pub signature_components: Vec<SignatureComponent<'a>>,
    pub signature: Vec<u8>,
    #[builder(default, setter(strip_option))]
    pub algorithm: Option<&'a str>,
    #[builder(default, setter(strip_option))]
    pub created: Option<SystemTime>,
    #[builder(default, setter(strip_option))]
    pub expires: Option<SystemTime>,
}

impl<'a> SignatureHeader<'a> {
    pub fn builder() -> SignatureHeaderBuilder<'a> {
        SignatureHeaderBuilder::default()
    }

    pub fn parse(raw: &'a str) -> Result<Self> {
        let kv_pairs = raw.split(',').filter_map(|pair| {
            let (key, value) = pair.split_once('=')?;
            Some((key.trim(), value.trim().trim_matches('"')))
        });

        let mut builder = Self::builder();
        for (key, value) in kv_pairs {
            match key {
                "keyId" => builder.key_id(value),
                "signature" => builder.signature(general_purpose::STANDARD.decode(value)?),
                "headers" => {
                    let components = value
                        .split_whitespace()
                        .map(SignatureComponent::parse)
                        .collect::<Result<Vec<_>, _>>()?;

                    builder.signature_components(components)
                }
                "algorithm" => builder.algorithm(value),
                "created" => builder.created(SystemTime::from_unix_timestamp(value.parse()?)),
                "expires" => builder.expires(SystemTime::from_unix_timestamp(value.parse()?)),
                _ => continue,
            };
        }

        builder.build().map_err(Error::from)
    }

    pub fn components_include_date_header(&self) -> bool {
        self.signature_components
            .iter()
            .any(|component| component.as_str().to_lowercase() == "date")
    }

    /// Check if the signature is expired by checking the `(expires)` pseudo-header
    ///
    /// If the header isn't present, just default to considering the signature valid
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires {
            expires < SystemTime::now()
        } else {
            false
        }
    }

    /// Check if the signature is expired really strictly
    ///
    /// Try to read one of the signed "created at" timestamps
    /// If the signature doesn't contain any signed timestamps, reject the signature as invalid.
    ///
    /// The signature is considered expired if
    ///
    /// - When the `(expires)` pseudo-header tells us it is expired
    /// - Our implementation considers the signature expired according to the setting
    ///
    /// Whichever is true first
    pub fn is_expired_strict(&self, parts: &Parts, enforced_duration: Duration) -> Result<bool> {
        let signed_created = if self
            .signature_components
            .contains(&SignatureComponent::Created)
        {
            self.created.ok_or(Error::InvalidSignatureHeader)?.into()
        } else if self.components_include_date_header() {
            let date_header = parts
                .headers
                .get(DATE)
                .ok_or(Error::InvalidSignatureHeader)?;

            OffsetDateTime::parse(date_header.to_str()?, &Rfc2822)?
        } else {
            return Err(Error::InvalidSignatureHeader);
        };

        // Determine when the signature is considered expired
        let expires_at: OffsetDateTime = {
            let enforced_expires_at = signed_created + time::Duration::try_from(enforced_duration)?;

            self.expires.map_or(enforced_expires_at, |expires| {
                min(expires.into(), enforced_expires_at)
            })
        };

        Ok(OffsetDateTime::now_utc() >= expires_at)
    }
}

impl TryFrom<SignatureHeader<'_>> for String {
    type Error = SystemTimeError;

    fn try_from(value: SignatureHeader<'_>) -> Result<Self, Self::Error> {
        let signature = general_purpose::STANDARD.encode(value.signature);
        let headers = value
            .signature_components
            .iter()
            .map(SignatureComponent::as_str)
            .collect::<Vec<_>>()
            .join(" ");

        let mut signature_header = format!(
            "keyId=\"{}\",signature=\"{signature}\",headers=\"{headers}\"",
            value.key_id
        );

        if let Some(algorithm) = value.algorithm {
            signature_header.push_str(",algorithm=\"");
            signature_header.push_str(algorithm);
            signature_header.push('"');
        }

        if let Some(created) = value.created {
            signature_header.push_str(",created=");
            signature_header.push_str(&created.to_unix_timestamp()?.to_string());
        }

        if let Some(expires) = value.expires {
            signature_header.push_str(",expires=");
            signature_header.push_str(&expires.to_unix_timestamp()?.to_string());
        }

        Ok(signature_header)
    }
}

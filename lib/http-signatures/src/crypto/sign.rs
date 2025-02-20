use ring::{
    rand::SystemRandom,
    signature::{Ed25519KeyPair, RSA_PKCS1_SHA256, RsaKeyPair, Signature},
};

/// Signing key definition
pub trait SigningKey {
    /// Type the signature algorithm outputs
    type Output: AsRef<[u8]>;

    /// Sign a message
    fn sign(&self, msg: &[u8]) -> Self::Output;
}

impl SigningKey for Ed25519KeyPair {
    type Output = Signature;

    #[inline]
    fn sign(&self, msg: &[u8]) -> Self::Output {
        self.sign(msg)
    }
}

impl SigningKey for RsaKeyPair {
    type Output = Vec<u8>;

    #[inline]
    fn sign(&self, msg: &[u8]) -> Self::Output {
        let mut buf = vec![0; self.public().modulus_len()];

        let rng = SystemRandom::new();
        self.sign(&RSA_PKCS1_SHA256, &rng, msg, &mut buf)
            .expect("Failed to sign message");

        buf
    }
}

/// Sign a message with the provided signing key and encode the returned signature in Base64
#[inline]
pub fn sign<SK>(payload: &[u8], key: &SK) -> String
where
    SK: SigningKey,
{
    base64_simd::STANDARD.encode_to_string(key.sign(payload))
}

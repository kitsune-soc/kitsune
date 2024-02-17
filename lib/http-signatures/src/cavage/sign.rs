use ring::{
    rand::SystemRandom,
    signature::{Ed25519KeyPair, RsaKeyPair, Signature, RSA_PKCS1_SHA256},
};

pub trait SigningKey {
    type Output: AsRef<[u8]>;

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

#[inline]
pub fn sign<KP>(payload: &[u8], key: &KP) -> String
where
    KP: SigningKey,
{
    base64_simd::STANDARD.encode_to_string(key.sign(payload))
}

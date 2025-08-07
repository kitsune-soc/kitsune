#![allow(dead_code)]

use const_oid::db::rfc5912::RSA_ENCRYPTION;
use http::{Method, Request, Uri};
use pkcs8::{
    Document, PrivateKeyInfo, SecretDocument, SubjectPublicKeyInfoRef, der::Encode,
    spki::AlgorithmIdentifier,
};
use ring::signature::{
    RSA_PKCS1_1024_8192_SHA256_FOR_LEGACY_USE_ONLY, RsaKeyPair, UnparsedPublicKey,
};

const PUBLIC_KEY: &str = r"
-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDCFENGw33yGihy92pDjZQhl0C3
6rPJj+CvfSC8+q28hxA161QFNUd13wuCTUcq0Qd2qsBe/2hFyc2DCJJg0h1L78+6
Z4UMR7EOcpfdUE9Hf3m/hs+FUR45uBJeDK1HSFHD8bHKD6kv8FPGfJTotc+2xjJw
oYi+1hqp1fIekaxsyQIDAQAB
-----END PUBLIC KEY-----
";

const SOME_PRIVATE_KEY: &str = r"
-----BEGIN RSA PRIVATE KEY-----
MIIEogIBAAKCAQEAtVpWdypmE7PzU4IsR4KOOKCKfDzsF1PDxwpkfFof7kcHGzNo
QC8b8yUGNHF5YYhzGR2FM/sNBdXroZvPJ1FLAE/pfm1TFLArPubzR/pan6/8uX6K
SBwECUblmuF4NpZQ2yj0fIjroe33UlBVW85b1xLiEpgKr/cMHnAoFhY7xuGoafez
1wDym9oGPAaMM9+2VVXXC9UYMNQAOf75/mRHSRsLkxjTE0K2rMufAIAYsnkZ/UlK
nqBZrR5dnHO87NgG46W8zqMUIm+OmjrCI2A+LSpjMz+09iZIoxj2QxbAz08S6dsu
PJz5NXVt6eISBsQkN5YAqpLfqdyVzcme5AcZsQIDAQABAoIBADgL6Tj+03k3XfXq
/wBCqu15QBNRSK2U56Z14cREniWnsdyIMKnVZU/pm1dN0nOAFgInC2mbJtUs3Zue
aZD/IKzCBala5Bg6scLI4VrXVnaPjw1MvDO45M5xKLiLAfnszqRMrfJm5luvDYZU
6WxsBBod7dxNPNBRC1/Ezb61tFesFIA/04VURbJ0dGQJ0Rp6nfOf5kdAz/+TxaKA
PeCveTRc1neESOAvNHMAX9kbfaB96FKeVoYy2DmWf8giXd5bt7YwhRwQj82XWDLu
kAfjZoqEUixz/vsgZ5+3AayZGAEOc9OvuQATs7wSCPyreeuSVqDINpOTqGhHJMGC
HSYvxZkCgYEA3hu3dDnyaww72zci/7iLzsWjo0NBSK+6ZRSd2Y+Oj1UNN/Hw2P9j
G74RU+q0ZjwbVuleD1LPC4XGWdtOxnCexykfkOotvsRLDtlOY1ABHIEiwwmzV3Mm
ByWy8rsa/w6V8ItBScYLE7xrMXYmoZqZ+6pCMYo8Ni/ED7mrucEOEoMCgYEA0QaU
u9HXgnIKH7AZwffhdhS9uw/ZqnC/WEeb0dQ2Lzi2DDMIEeJtQq5baR+C2/IC8yBf
gBlEhXveE7KOeid38JtNOhEHf4F+SuRN4mwWxxk5VzKLo6wC4BaoJrl4THThavGU
JTr6gMojRjNqqllAtGgHwLhQCgShUgVePzod3LsCgYBJqFtwmf8A7S+0hVaAA82p
pvWboSQ3XL+t4eZvTiJy+jvF/+BltlxByQiqEb394ZUXf5EH9+hd4+Fhz08SlCqz
1bl4L5E4IJTbuck7Oj8EGvdSQxdMuw0zdZcg1Fghmc6z1Rqzwo/N3cCWyJ4LHeBP
C6mkEDnjpneY13RRx+pIzQKBgGPJp4HO2PqeZLTiBjnyk8Eif71pALn8n2yOqxXO
IJwEj+xdBHI9TXny8RLLh1ZnP/8/qjfmWC79hnSS3q/0Xa8RBRo+fPzjHh60xXXd
sjYUlapKKB3YBXtjdZ0fGA4wEllSwS3Q7TxEw/hEnZx7hYdazrCzjQprUXRtuaOn
pA/3AoGAVmLOtGCN206G3vTg5ftUqzr+/2Nz4veAyI/FvXJpiGh8JzqHr1Y1EifE
qkTAhsrR20WzWEscInV4gb5Q8SuAzhREZ6CJZnw1uRuzqJlJvc4h8Bsd8rNcZSNJ
ycN7jXCNeRs5qIcy7Dej1Exzu0+Qvn4mzf1iFEAxPHHlzXQ+UMs=
-----END RSA PRIVATE KEY-----
";

#[must_use]
pub fn get_request() -> Request<()> {
    Request::builder()
        .method(Method::POST)
        .uri(Uri::from_static("/foo?param=value&pet=dog"))
        .header("Host", "example.com")
        .header("Date", "Sun, 05 Jan 2014 21:31:40 GMT")
        .header("Content-Type", "application/json")
        .header(
            "Digest",
            "SHA-256=X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=",
        )
        .header("Content-Length", "18")
        .body(())
        .unwrap()
}

#[must_use]
pub fn get_pkcs8_private_key() -> Vec<u8> {
    let (_tag, document) = SecretDocument::from_pem(SOME_PRIVATE_KEY).unwrap();
    let private_key_info = PrivateKeyInfo {
        algorithm: AlgorithmIdentifier {
            oid: RSA_ENCRYPTION,
            parameters: None,
        },
        private_key: document.as_bytes(),
        public_key: None,
    };
    private_key_info.to_der().unwrap()
}

#[must_use]
pub fn get_private_key() -> RsaKeyPair {
    let (_tag, document) = SecretDocument::from_pem(SOME_PRIVATE_KEY).unwrap();
    RsaKeyPair::from_der(document.as_bytes()).unwrap()
}

#[must_use]
pub fn get_public_key() -> UnparsedPublicKey<Vec<u8>> {
    let (_tag, pub_key) = Document::from_pem(PUBLIC_KEY).unwrap();
    let pub_key: SubjectPublicKeyInfoRef<'_> = pub_key.decode_msg().unwrap();
    let pub_key = pub_key.subject_public_key.raw_bytes().to_vec();

    UnparsedPublicKey::new(&RSA_PKCS1_1024_8192_SHA256_FOR_LEGACY_USE_ONLY, pub_key)
}

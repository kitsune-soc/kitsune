use komainu::scope::Scope;
use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, Mutex},
};

fn define_client(client_name: &str, scopes: Scope) -> komainu::Client<'static> {
    komainu::Client {
        client_id: Cow::Owned(client_name.into()),
        client_secret: Cow::Owned(format!("{client_name}_sec")),
        scopes,
        redirect_uri: Cow::Owned(format!("http://{client_name}.example")),
    }
}

#[derive(Clone)]
pub struct ClientExtractor {
    clients: Arc<Mutex<HashMap<String, komainu::Client<'static>>>>,
}

impl ClientExtractor {
    pub fn empty() -> Self {
        Self {
            clients: Arc::default(),
        }
    }

    fn insert(&self, client: komainu::Client<'static>) {
        let mut guard = self.clients.lock().unwrap();
        guard.insert(client.client_id.clone().into_owned(), client);
    }
}

impl Default for ClientExtractor {
    fn default() -> Self {
        let extractor = Self::from_iter(
            [
                ("client_1", Scope::from_iter(["read", "write"])),
                ("client_2", Scope::from_iter(["follow", "push"])),
                ("client_3", Scope::new()),
            ]
            .map(|(client_name, scopes)| define_client(client_name, scopes)),
        );

        extractor.insert(komainu::Client {
            client_id: "malicious_client_1".into(),
            client_secret: "malicious_client_1_sec".into(),
            scopes: Scope::from_iter(["read", "write"]),
            redirect_uri: "http://client_1.example".into(),
        });

        extractor
    }
}

impl FromIterator<komainu::Client<'static>> for ClientExtractor {
    fn from_iter<T: IntoIterator<Item = komainu::Client<'static>>>(iter: T) -> Self {
        iter.into_iter().fold(Self::empty(), |acc, item| {
            acc.insert(item);
            acc
        })
    }
}

impl komainu::ClientExtractor for ClientExtractor {
    async fn extract(
        &self,
        client_id: &str,
        client_secret: Option<&str>,
    ) -> Result<komainu::Client<'_>, komainu::error::Error> {
        let guard = self.clients.lock().unwrap();

        let client = guard.get(client_id).unwrap();
        if let Some(client_secret) = client_secret {
            assert_eq!(client.client_secret, client_secret);
        }

        Ok(client.clone())
    }
}

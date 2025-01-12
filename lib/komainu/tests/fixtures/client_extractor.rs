use komainu::scope::Scope;
use std::{borrow::Cow, collections::HashMap};

fn define_client(client_name: &str, scopes: Scope) -> komainu::Client<'static> {
    komainu::Client {
        client_id: Cow::Owned(client_name.into()),
        client_secret: Cow::Owned(format!("{client_name}_sec")),
        scopes,
        redirect_uri: Cow::Owned(format!("http://{client_name}.example")),
    }
}

pub struct ClientExtractor {
    clients: HashMap<String, komainu::Client<'static>>,
}

impl ClientExtractor {
    pub fn empty() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    fn insert(&mut self, client: komainu::Client<'static>) {
        self.clients
            .insert(client.client_id.clone().into_owned(), client);
    }
}

impl Default for ClientExtractor {
    fn default() -> Self {
        Self::from_iter(
            [
                ("client_1", Scope::from_iter(["read", "write"])),
                ("client_2", Scope::from_iter(["follow", "push"])),
                ("client_3", Scope::new()),
            ]
            .map(|(client_name, scopes)| define_client(client_name, scopes)),
        )
    }
}

impl FromIterator<komainu::Client<'static>> for ClientExtractor {
    fn from_iter<T: IntoIterator<Item = komainu::Client<'static>>>(iter: T) -> Self {
        iter.into_iter().fold(Self::empty(), |mut acc, item| {
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
        let client = self.clients.get(client_id).unwrap();
        if let Some(client_secret) = client_secret {
            assert_eq!(client.client_secret, client_secret);
        }

        Ok(client.clone())
    }
}

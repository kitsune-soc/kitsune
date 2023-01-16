use crate::{
    db::model::{account, post},
    error::Result,
};
use futures_util::stream;
use http::Uri;
use kitsune_search_proto::{
    common::SearchIndex,
    index::{
        add_index_request::IndexEntity, index_client::IndexClient, AddAccountIndex,
        AddIndexRequest, AddPostIndex, RemoveIndexRequest, ResetRequest,
    },
    search::{search_client::SearchClient, SearchRequest, SearchResponse},
};
use std::{future, str::FromStr};
use tonic::transport::Channel;

pub enum SearchItem {
    Account(account::Model),
    Post(post::Model),
}

impl From<account::Model> for SearchItem {
    fn from(account: account::Model) -> Self {
        Self::Account(account)
    }
}

impl From<post::Model> for SearchItem {
    fn from(post: post::Model) -> Self {
        Self::Post(post)
    }
}

#[derive(Clone)]
pub struct SearchService {
    searcher: SearchClient<Channel>,
    indexer: IndexClient<Channel>,
}

impl SearchService {
    pub async fn new(endpoint: &str) -> Result<Self> {
        let channel = Channel::builder(Uri::from_str(endpoint)?).connect().await?;

        Ok(Self {
            searcher: SearchClient::new(channel.clone()),
            indexer: IndexClient::new(channel),
        })
    }

    pub async fn add_to_index<I>(&mut self, item: I) -> Result<()>
    where
        I: Into<SearchItem>,
    {
        let request = match item.into() {
            SearchItem::Account(account) => AddIndexRequest {
                index_entity: Some(IndexEntity::Account(AddAccountIndex {
                    id: account.id.as_bytes().to_vec(),
                    display_name: account.display_name,
                    username: account.username,
                    description: account.note,
                })),
            },
            SearchItem::Post(post) => AddIndexRequest {
                index_entity: Some(IndexEntity::Post(AddPostIndex {
                    id: post.id.as_bytes().to_vec(),
                    subject: post.subject,
                    content: post.content,
                })),
            },
        };

        self.indexer
            .add(stream::once(future::ready(request)))
            .await?;

        Ok(())
    }

    pub async fn remove_from_index<I>(&mut self, item: I) -> Result<()>
    where
        I: Into<SearchItem>,
    {
        let request = match item.into() {
            SearchItem::Account(account) => RemoveIndexRequest {
                index: SearchIndex::Account.into(),
                id: account.id.as_bytes().to_vec(),
            },
            SearchItem::Post(post) => RemoveIndexRequest {
                index: SearchIndex::Post.into(),
                id: post.id.as_bytes().to_vec(),
            },
        };

        self.indexer
            .remove(stream::once(future::ready(request)))
            .await?;

        Ok(())
    }

    pub async fn reset_index(&mut self, index: SearchIndex) -> Result<()> {
        let request = ResetRequest {
            index: index.into(),
        };
        self.indexer.reset(request).await?;

        Ok(())
    }

    pub async fn search(
        &mut self,
        index: SearchIndex,
        query: String,
        page: Option<u64>,
    ) -> Result<SearchResponse> {
        let request = SearchRequest {
            index: index.into(),
            query,
            page: page.unwrap_or(0),
        };

        Ok(self.searcher.search(request).await?.into_inner())
    }
}

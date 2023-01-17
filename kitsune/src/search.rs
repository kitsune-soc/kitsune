use crate::{
    db::model::{account, post},
    error::Result,
};
use futures_util::stream;
use kitsune_search_proto::{
    common::SearchIndex,
    index::{
        add_index_request::IndexEntity, index_client::IndexClient, AddAccountIndex,
        AddIndexRequest, AddPostIndex, RemoveIndexRequest, ResetRequest,
    },
    search::{search_client::SearchClient, IndexPagination, SearchRequest, SearchResponse},
};
use std::future;
use tonic::transport::{Channel, Endpoint};
use uuid::Uuid;

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
    pub async fn new(index_endpoint: &str, search_endpoints: &[String]) -> Result<Self> {
        let index_channel = Endpoint::from_shared(index_endpoint.to_string())?
            .connect()
            .await?;

        let search_endpoints: Vec<Endpoint> = search_endpoints
            .iter()
            .cloned()
            .map(Endpoint::from_shared)
            .collect::<Result<_, tonic::transport::Error>>()?;
        let search_channel = Channel::balance_list(search_endpoints.into_iter());

        Ok(Self {
            searcher: SearchClient::new(search_channel),
            indexer: IndexClient::new(index_channel),
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
        max_results: u64,
        offset: u64,
        min_id: Option<Uuid>,
        max_id: Option<Uuid>,
    ) -> Result<SearchResponse> {
        let request = SearchRequest {
            index: index.into(),
            query,
            max_results,
            offset,
            indices: Some(IndexPagination {
                max_id: min_id.as_ref().map(|id| id.as_bytes().to_vec()),
                min_id: max_id.as_ref().map(|id| id.as_bytes().to_vec()),
            }),
        };

        Ok(self.searcher.search(request).await?.into_inner())
    }
}

use self::common::TestClient;
use futures_util::stream;
use kitsune_search_proto::{
    common::SearchIndex,
    index::{add_index_request::IndexEntity, AddAccountIndex, AddIndexRequest, RemoveIndexRequest},
    search::SearchRequest,
};
use std::{future, time::Duration};

mod common;

#[tokio::test]
async fn index_search_remove() {
    let mut test_client = TestClient::create().await;

    let id: [u8; 24] = rand::random();
    test_client
        .index
        .add(stream::once(future::ready(AddIndexRequest {
            index_entity: Some(IndexEntity::Account(AddAccountIndex {
                id: id.to_vec(),
                display_name: Some("name".into()),
                username: "cool_username".into(),
                description: Some("Really cool test account. Very important".into()),
            })),
        })))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await; // Wait until the write as propagated to the reader

    let response = test_client
        .search
        .search(SearchRequest {
            index: SearchIndex::Account.into(),
            query: "tset".into(),
            page: 0,
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(response.result.len(), 1);
    assert_eq!(response.result[0].id, id);

    // -- Remove the account from the index --

    test_client
        .index
        .remove(stream::once(future::ready(RemoveIndexRequest {
            index: SearchIndex::Account.into(),
            id: id.to_vec(),
        })))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await; // Wait until the write as propagated to the reader

    let response = test_client
        .search
        .search(SearchRequest {
            index: SearchIndex::Account.into(),
            query: "tset".into(),
            page: 0,
        })
        .await
        .unwrap()
        .into_inner();

    assert!(response.result.is_empty(), "{response:#?}");
}

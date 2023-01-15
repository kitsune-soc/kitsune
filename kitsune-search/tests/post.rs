use self::common::TestClient;
use futures_util::stream;
use kitsune_search_proto::{
    common::SearchIndex,
    index::{add_index_request::IndexData, AddIndexRequest, AddPostIndex, RemoveIndexRequest},
    search::SearchRequest,
};
use std::{future, time::Duration};

mod common;

const POST_CONTENT: &str = r#"
Lorem ipsum dolor sit amet, consectetur adipiscing elit. 
Fusce hendrerit consequat tellus sed rhoncus. Nullam nec ultrices tellus. 
In vitae tempus mauris, vel auctor orci. 
In porttitor, lectus sed consectetur blandit, ligula neque efficitur eros, sit amet ullamcorper nunc odio eu sem. 
Quisque luctus rutrum ullamcorper. 
Nam vitae varius turpis, eget porta enim. 
Vestibulum luctus ex id ipsum mattis porttitor. 
Duis faucibus risus quis varius cursus. 
Praesent et mi orci. 
Curabitur id fermentum elit, sed congue lacus. 
Pellentesque id augue vitae tortor vehicula placerat vestibulum et justo.

Praesent quis ex magna. Ut congue dapibus tortor quis dignissim. 
Vestibulum sit amet nulla faucibus, faucibus felis in, ultrices odio. 
Cras eu tellus at eros molestie condimentum. 
Phasellus ornare est a ante blandit, eget commodo odio dapibus. 
Praesent enim mauris, consectetur quis rutrum quis, congue sit amet ligula. 
Vestibulum finibus tincidunt ipsum, id luctus lorem tincidunt nec. 
Duis nec arcu at libero finibus finibus eget a elit. 
Nullam a massa ornare nisl tristique blandit vitae vitae nunc. 
Suspendisse faucibus pellentesque risus, id tincidunt urna vehicula a. 
Maecenas at justo auctor metus varius ultrices. 
Nullam feugiat dictum tortor, ac vestibulum dolor bibendum nec. 
Phasellus varius vehicula lectus. 
Duis vestibulum nisi turpis, et pharetra massa sollicitudin tempor. 
Vivamus risus dolor, venenatis a convallis eget, tincidunt sed lacus. 
Sed tincidunt eros gravida, sollicitudin enim ut, pellentesque libero.
"#;

#[tokio::test]
async fn index_search_remove() {
    let mut test_client = TestClient::create().await;

    let id: [u8; 24] = rand::random();
    test_client
        .index
        .add(stream::once(future::ready(AddIndexRequest {
            index_data: Some(IndexData::Post(AddPostIndex {
                id: id.to_vec(),
                subject: None,
                content: POST_CONTENT.to_string(),
            })),
        })))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;

    let response = test_client
        .search
        .search(SearchRequest {
            index: SearchIndex::Post.into(),
            query: "lroem".into(),
            page: 0,
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(response.result.len(), 1);
    assert_eq!(response.result[0].id, id);

    // -- Remove post from the index --

    test_client
        .index
        .remove(stream::once(future::ready(RemoveIndexRequest {
            index: SearchIndex::Post.into(),
            id: id.to_vec(),
        })))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;

    let response = test_client
        .search
        .search(SearchRequest {
            index: SearchIndex::Post.into(),
            query: "lroem".into(),
            page: 0,
        })
        .await
        .unwrap()
        .into_inner();

    assert!(response.result.is_empty());
}

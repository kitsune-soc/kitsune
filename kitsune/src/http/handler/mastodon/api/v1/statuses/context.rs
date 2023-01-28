use crate::{
    db::model::post::{self, Visibility},
    error::Result,
    mapping::IntoMastodon,
    state::Zustand,
};
use async_recursion::async_recursion;
use axum::{
    debug_handler,
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use kitsune_type::mastodon::{status::Context, Status};
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};
use std::collections::VecDeque;
use uuid::Uuid;

#[async_recursion]
async fn get_ancestors(
    state: &Zustand,
    post: &post::Model,
    ancestors: &mut VecDeque<Status>,
) -> Result<()> {
    if let Some(in_reply_to) = post
        .find_linked(post::InReplyTo)
        .filter(post::Column::Visibility.is_in([Visibility::Public, Visibility::Unlisted]))
        .one(&state.db_conn)
        .await?
    {
        ancestors.push_front(in_reply_to.clone().into_mastodon(state).await?);
        get_ancestors(state, &in_reply_to, ancestors).await?;
    }

    Ok(())
}

#[async_recursion]
async fn get_descendants(
    state: &Zustand,
    post: &post::Model,
    descendants: &mut Vec<Status>,
) -> Result<()> {
    let subdescendants = post::Entity::find()
        .filter(post::Column::InReplyToId.eq(post.id))
        .filter(post::Column::Visibility.is_in([Visibility::Public, Visibility::Unlisted]))
        .all(&state.db_conn)
        .await?;

    for descendant in subdescendants {
        descendants.push(descendant.clone().into_mastodon(state).await?);
        get_descendants(state, &descendant, descendants).await?;
    }

    Ok(())
}

#[debug_handler]
pub async fn get(State(state): State<Zustand>, Path(id): Path<Uuid>) -> Result<Response> {
    let Some(genesis) = post::Entity::find_by_id(id).one(&state.db_conn).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let mut ancestors = VecDeque::new();
    get_ancestors(&state, &genesis, &mut ancestors).await?;

    let mut descendants = Vec::new();
    get_descendants(&state, &genesis, &mut descendants).await?;

    Ok(Json(Context {
        ancestors,
        descendants,
    })
    .into_response())
}

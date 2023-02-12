use crate::{
    error::Result,
    http::extractor::{AuthExtractor, FormOrJson, MastodonAuthExtractor},
    job::{deliver::delete::DeleteDeliveryContext, Job},
    mapping::IntoMastodon,
    service::{post::CreatePost, search::SearchService},
    state::Zustand,
};
use axum::{
    debug_handler,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use chrono::Utc;
use http::StatusCode;
use kitsune_db::{
    custom::{JobState, Role, Visibility},
    entity::{
        jobs, posts,
        prelude::{Posts, UsersRoles},
        users_roles,
    },
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait, PaginatorTrait,
    QueryFilter,
};
use serde::Deserialize;
use uuid::Uuid;

mod context;
mod favourite;
mod unfavourite;

#[derive(Deserialize)]
struct CreateForm {
    status: String,
    in_reply_to_id: Option<Uuid>,
    #[serde(default)]
    sensitive: bool,
    spoiler_text: Option<String>,
    #[serde(default)]
    visibility: Visibility,
}

#[debug_handler(state = Zustand)]
async fn delete(
    State(state): State<Zustand>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    let Some(post) = Posts::find_by_id(id).one(&state.db_conn).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    if post.account_id != user_data.account.id {
        let admin_role_count = user_data
            .user
            .find_related(UsersRoles)
            .filter(users_roles::Column::Role.eq(Role::Administrator))
            .count(&state.db_conn)
            .await?;

        if admin_role_count == 0 {
            return Ok(StatusCode::UNAUTHORIZED.into_response());
        }
    }

    let job_context = Job::DeliverDelete(DeleteDeliveryContext { post_id: post.id });
    jobs::Model {
        id: Uuid::now_v7(),
        state: JobState::Queued,
        run_at: Utc::now().into(),
        context: serde_json::to_value(job_context).unwrap(),
        fail_count: 0,
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    }
    .into_active_model()
    .insert(&state.db_conn)
    .await?;

    state.service.search.remove_from_index(post).await?;

    Ok(StatusCode::OK.into_response())
}

#[debug_handler(state = Zustand)]
async fn get(
    State(state): State<Zustand>,
    _user_data: Option<MastodonAuthExtractor>,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    let Some(post) = Posts::find()
        .filter(posts::Column::Id.eq(id))
        .filter(
            posts::Column::Visibility
                .eq(Visibility::Public)
                .or(posts::Column::Visibility.eq(Visibility::Unlisted))
        )
        .one(&state.db_conn)
        .await?
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(Json(post.into_mastodon(&state).await?).into_response())
}

#[debug_handler(state = Zustand)]
async fn post(
    State(state): State<Zustand>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    FormOrJson(form): FormOrJson<CreateForm>,
) -> Result<Response> {
    // TODO: Use the post service
    let mut create_post = CreatePost::builder()
        .author_id(user_data.account.id)
        .content(form.status)
        .sensitive(form.sensitive)
        .visibility(form.visibility)
        .clone();

    if let Some(subject) = form.spoiler_text {
        create_post.subject(subject);
    }
    if let Some(in_reply_to_id) = form.in_reply_to_id {
        create_post.in_reply_to_id(in_reply_to_id);
    }

    let status = state
        .service
        .post
        .create(create_post.build().unwrap())
        .await?
        .into_mastodon(&state)
        .await?;

    Ok(Json(status).into_response())
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/", routing::post(post))
        .route("/:id", routing::get(get).delete(delete))
        .route("/:id/context", routing::get(context::get))
        .route("/:id/favourite", routing::post(favourite::post))
        .route("/:id/unfavourite", routing::post(unfavourite::post))
}

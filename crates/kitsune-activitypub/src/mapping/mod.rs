use kitsune_db::PgPool;
use kitsune_service::attachment::AttachmentService;
use kitsune_service::url::UrlService;
use typed_builder::TypedBuilder;

mod activity;
mod object;
mod util;

pub use self::activity::IntoActivity;
pub use self::object::IntoObject;

#[derive(TypedBuilder)]
pub struct Service {
    attachment: AttachmentService,
    url: UrlService,
}

#[derive(TypedBuilder)]
pub struct State {
    db_pool: PgPool,
    service: Service,
}

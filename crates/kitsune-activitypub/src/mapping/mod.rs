use kitsune_db::PgPool;
use kitsune_service::attachment::AttachmentService;
use kitsune_url::UrlService;
use typed_builder::TypedBuilder;

mod activity;
mod object;
mod util;

pub use self::activity::IntoActivity;
pub use self::object::IntoObject;

#[derive(Clone, Copy, TypedBuilder)]
pub struct Service<'a> {
    attachment: &'a AttachmentService,
    url: &'a UrlService,
}

#[derive(Clone, Copy, TypedBuilder)]
pub struct State<'a> {
    db_pool: &'a PgPool,
    service: Service<'a>,
}

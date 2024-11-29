use arc_swap::ArcSwapAny;
use core::str;
use rust_embed::RustEmbed;
use std::sync::OnceLock;
use triomphe::Arc;

static ENVIRONMENT: OnceLock<ArcSwapAny<Arc<minijinja::Environment<'static>>>> = OnceLock::new();

#[derive(RustEmbed)]
#[folder = "templates"]
struct TemplateDir;

fn embed_loader(path: &str) -> Result<Option<String>, minijinja::Error> {
    let maybe_data = TemplateDir::get(path).map(|embedded_file| embedded_file.data);
    let maybe_template = maybe_data
        .map(|data| str::from_utf8(&data).map(ToString::to_string))
        .transpose()
        .map_err(|error| {
            minijinja::Error::new(minijinja::ErrorKind::CannotDeserialize, error.to_string())
        })?;

    Ok(maybe_template)
}

fn init_environment() -> minijinja::Environment<'static> {
    let mut environment = minijinja::Environment::new();
    environment.set_loader(embed_loader);
    environment
}

#[track_caller]
pub fn render<S>(name: &str, ctx: S) -> Option<String>
where
    S: serde::Serialize,
{
    let handle = ENVIRONMENT
        .get_or_init(|| {
            // ToDo: Spawn watcher on the path which replaces the environment through the arc-swap
            ArcSwapAny::new(Arc::new(init_environment()))
        })
        .load_full();

    let template = handle
        .get_template(name)
        .inspect_err(|error| error!(?error, "failed to get template"))
        .ok()?;

    let rendered = template.render(ctx).expect("failed to render template");

    Some(rendered)
}

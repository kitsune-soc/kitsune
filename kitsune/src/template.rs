use arc_swap::ArcSwapAny;
use core::str;
use notify_debouncer_full::{notify, DebounceEventResult};
use rust_embed::RustEmbed;
use std::{mem::ManuallyDrop, path::Path, sync::OnceLock, time::Duration};
use triomphe::Arc;

static ENVIRONMENT: OnceLock<ArcSwapAny<Arc<minijinja::Environment<'static>>>> = OnceLock::new();

#[derive(RustEmbed)]
#[folder = "templates"]
struct TemplateDir;

fn embed_loader(path: &str) -> Result<Option<String>, minijinja::Error> {
    let maybe_data = TemplateDir::get(path).map(|embedded_file| embedded_file.data);
    let maybe_template = maybe_data
        .map(|data| simdutf8::basic::from_utf8(&data).map(ToString::to_string))
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

fn spawn_watcher() {
    let watcher = notify_debouncer_full::new_debouncer(
        Duration::from_secs(1),
        None,
        |events: DebounceEventResult| {
            let Ok(events) = events else {
                return;
            };

            for event in events {
                if matches!(
                    event.event,
                    notify::Event {
                        kind: notify::EventKind::Create(..)
                            | notify::EventKind::Modify(..)
                            | notify::EventKind::Remove(..),
                        ..
                    }
                ) {
                    debug!(?event.paths, "reloading templates");

                    if let Some(env) = ENVIRONMENT.get() {
                        env.store(Arc::new(init_environment()));
                    }
                }
            }
        },
    )
    .unwrap();

    let mut watcher = ManuallyDrop::new(watcher);
    let template_dir = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates"));

    watcher
        .watch(template_dir, notify::RecursiveMode::Recursive)
        .unwrap();
}

#[track_caller]
pub fn render<S>(name: &str, ctx: S) -> Option<String>
where
    S: serde::Serialize,
{
    let handle = ENVIRONMENT
        .get_or_init(|| {
            #[cfg(debug_assertions)]
            spawn_watcher();
            ArcSwapAny::new(Arc::new(init_environment()))
        })
        .load();

    let template = handle
        .get_template(name)
        .inspect_err(|error| error!(?error, "failed to get template"))
        .ok()?;

    let rendered = template.render(ctx).expect("failed to render template");

    Some(rendered)
}

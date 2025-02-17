use crate::mrf_wit::v1::wasi::logging::logging::{self, Level};

macro_rules! event_dispatch {
    ($level:ident, $context:ident, $message:ident, {
        $($left:path => $right:path),+$(,)?
    }) => {{
        match $level {
            $(
                $left => event!($right, %$context, "{}", $message),
            )+
        }
    }};
}

impl logging::Host for crate::ctx::Context {
    async fn log(&mut self, level: Level, context: String, message: String) {
        event_dispatch!(level, context, message, {
            Level::Trace => tracing::Level::TRACE,
            Level::Debug => tracing::Level::DEBUG,
            Level::Info => tracing::Level::INFO,
            Level::Warn => tracing::Level::WARN,
            Level::Error => tracing::Level::ERROR,
            Level::Critical => tracing::Level::ERROR,
        });
    }
}

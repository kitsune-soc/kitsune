use super::{Deliverer, Fetcher, Resolver};
use paste::paste;
use triomphe::Arc;
use unsize::{CoerceUnsize, Coercion};

macro_rules! create_coerce {
    ($trait:ident) => {
        paste! {
            pub trait [<Coerce $trait>] {
                fn coerce(self) -> Arc<dyn $trait>;
            }

            impl<T> [<Coerce $trait>] for Arc<T> where T: $trait {
                fn coerce(self) -> Arc<dyn $trait> {
                    self.unsize(Coercion!(to dyn $trait))
                }
            }
        }
    };
}

create_coerce!(Deliverer);
create_coerce!(Fetcher);
create_coerce!(Resolver);

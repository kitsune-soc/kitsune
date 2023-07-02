/// Implements the [`ToRedisArgs`](::redis::ToRedisArgs) trait in a way that serialises the structure into
///
/// ```
/// [field name] [field value] [field name] [field value] ...
/// ```
#[macro_export]
macro_rules! impl_to_redis_args {
    (
        $(#[$top_annotation:meta])*
        $top_vis:vis struct $top_name:ident {
            $(
                $(#[$field_annotation:meta])*
                $field_name:ident : $field_ty:ty
            ),+
            $(,)?
        }
    ) => {
        $(#[$top_annotation])*
        $top_vis struct $top_name {
            $(
                $(#[$field_annotation:meta])*
                $field_name : $field_ty,
            )+
        }

        impl redis::ToRedisArgs for $top_name {
            fn write_redis_args<W>(&self, out: &mut W)
            where
                W: ?Sized + redis::RedisWrite
            {
                $(
                    redis::ToRedisArgs::write_redis_args(
                        &stringify!($field_name),
                        out,
                    );
                    redis::ToRedisArgs::write_redis_args(
                        &self.$field_name,
                        out,
                    );
                )+
            }

            fn is_single_arg(&self) -> bool {
                false
            }
        }
    };
}

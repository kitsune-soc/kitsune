use ahash::AHashMap;
use redis::{streams::StreamId, FromRedisValue, RedisResult};

#[derive(Clone, Debug)]
pub struct StreamAutoClaimReply {
    pub claimed_ids: Vec<StreamId>,
}

impl FromRedisValue for StreamAutoClaimReply {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Self> {
        type AutoClaimReturnValue = (
            String,
            Vec<Vec<(String, AHashMap<String, redis::Value>)>>,
            Vec<String>,
        );

        let (_start_stream_id, claimed_ids, _deleted_ids): AutoClaimReturnValue =
            redis::from_redis_value(v)?;

        let claimed_ids: Vec<StreamId> = claimed_ids
            .into_iter()
            .flat_map(|row| row.into_iter().map(|(id, map)| StreamId { id, map }))
            .collect();

        Ok(Self { claimed_ids })
    }
}

use ahash::AHashMap;
use redis::{streams::StreamId, FromRedisValue, RedisResult};

#[derive(Clone, Debug)]
pub struct StreamAutoClaimReply {
    pub start_stream_id: String,
    pub claimed_ids: Vec<StreamId>,
    pub deleted_ids: Vec<String>,
}

impl FromRedisValue for StreamAutoClaimReply {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Self> {
        type AutoClaimReturnValue = (
            String,
            Vec<Vec<(String, AHashMap<String, redis::Value>)>>,
            Vec<String>,
        );

        let (start_stream_id, claimed_ids, deleted_ids): AutoClaimReturnValue =
            redis::from_redis_value(v)?;

        let claimed_ids: Vec<StreamId> = claimed_ids
            .into_iter()
            .flat_map(|row| row.into_iter().map(|(id, map)| StreamId { id, map }))
            .collect();

        Ok(Self {
            start_stream_id,
            claimed_ids,
            deleted_ids,
        })
    }
}

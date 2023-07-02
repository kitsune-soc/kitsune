-- KEYS[1]: Name of the stream we want to put the ready jobs on
-- KEYS[2]: Name of the sorted set with scheduled jobs
--
-- The score of the values in the scheduled set is the timestamp at which the job gets ready for execution

local function getTimestamp()
    local redisTime = redis.call("TIME")
    return redisTime[1]
end

local function unpackObject(obj)
    local arr = {}
    for k, v in pairs(obj) do
        table.insert(arr, k)
        table.insert(arr, v)
    end

    return unpack(arr)
end

local jobStream = KEYS[1]
local scheduledJobSet = KEYS[2]

local readyJobs = redis.call("ZRANGE", scheduledJobSet, 0, getTimestamp(), "BYSCORE")

if #readyJobs > 0 then
    redis.call("ZREM", scheduledJobSet, unpack(readyJobs))
end

for _, rawJobData in ipairs(readyJobs) do
    local jobData = cjson.decode(rawJobData)
    redis.call("XADD", jobStream, "*", unpackObject(jobData))
end

# Job Scheduler

Kitsune uses the database to store and retrieve jobs that have to be run. 
There are options to tune the job scheduler to your specific needs.

```toml
[job-queue]
redis-url = "redis://localhost"
num-workers = 20
```

## `redis-url`

This option configures the Redis instance that the jobs are put on.

We utilize Redis streams and transactional Lua scripting to ensure no jobs are lost even in the case of a crash.

## `job-workers`

This option configures how many jobs are run concurrently at the same time.  

Each job is a lightweight task inside Kitsune's async runtime, so you can raise this well above what you'd raise a thread limit to.

> **Caution**: Each activity delivery is a job, and each of the delivery jobs run multiple connections concurrently. 
> Raising this job worker limit too high without also increasing the maximum file descriptors might lead to weird issues.

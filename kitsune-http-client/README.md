# kitsune-http-client

Opinionated HTTP client based on `hyper` and `tower-http`

## Why?

`reqwest` is a fine choice for a lot of applications. For our specific use-case however, we need something where we can have some more low-level control over the client.  
As an ActivityPub server, we are connecting to a lot of untrusted URLs and fetch data from it. 
`reqwest` doesn't let us easily limit the amount of bytes read from the remote server on the body layer.  
Combined with the absent `Content-Type` header, this is just a DoS waiting to happen.

Also we can integrate our HTTP signatures library more closely and ergonomically.

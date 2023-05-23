let Oidc = ./oidc.dhall

in  { frontend_dir : Text
    , job_workers : Natural
    , max_upload_size : Natural
    , media_proxy_enabled : Bool
    , oidc : Optional Oidc
    , port : Natural
    , prometheus_port : Natural
    , request_timeout_sec : Natural
    }

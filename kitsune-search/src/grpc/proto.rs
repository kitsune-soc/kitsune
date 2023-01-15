pub mod common {
    tonic::include_proto!("kitsune.common");
}

pub mod index {
    tonic::include_proto!("kitsune.index");
}

pub mod search {
    tonic::include_proto!("kitsune.search");
}

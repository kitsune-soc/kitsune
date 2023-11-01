use time::OffsetDateTime;

fn main() {
    let uuid = masto_id_convert::process("110368129515784116").unwrap();
    println!("Converted UUID: {uuid}");
    println!("UUID version: {}", uuid.get_version_num());
    println!("UUID variant: {}", uuid.get_variant());

    let (timestamp_seconds, timestamp_nanos) = uuid.get_timestamp().unwrap().to_unix();
    let timestamp_nanos = ((timestamp_seconds as i128) * 1_000_000_000) + (timestamp_nanos as i128);
    let uuid_timestamp = OffsetDateTime::from_unix_timestamp_nanos(timestamp_nanos).unwrap();

    println!("Embedded timestamp: {uuid_timestamp}");
}

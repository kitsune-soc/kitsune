#[cfg(target_pointer_width = "64")]
pub use sonic_rs::{from_reader, from_slice, from_str, to_string, Result};

#[inline]
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: std::io::Write,
    T: ?Sized + serde::Serialize,
{
    use sonic_rs::writer::BufferedWriter;

    sonic_rs::to_writer(BufferedWriter::new(writer), value)
}

#[cfg(not(target_pointer_width = "64"))]
pub use serde_json::{from_reader, from_slice, from_str, to_string, to_writer, Result};

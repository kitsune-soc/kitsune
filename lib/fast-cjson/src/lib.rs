use crate::util::Either;
use icu_normalizer::ComposingNormalizer;
use memchr::memchr3;
use serde::Serialize;
use sonic_rs::{
    Serializer,
    format::{CompactFormatter, Formatter},
    writer::{BufferedWriter, WriteExt},
};
use std::{
    collections::BTreeMap,
    io::{self, Write as _},
    mem,
};

mod util;

#[derive(Debug)]
enum Collecting {
    Key(Vec<u8>),
    Value { key: Vec<u8>, value: Vec<u8> },
}

impl Default for Collecting {
    fn default() -> Self {
        Self::Key(Vec::new())
    }
}

#[derive(Debug, Default)]
struct Object {
    obj: BTreeMap<Vec<u8>, Vec<u8>>,
    state: Collecting,
}

/// A [`Formatter`](sonic_rs::format::Formatter) that produces canonical JSON.
#[derive(Debug, Default)]
pub struct CanonicalFormatter {
    object_stack: Vec<Object>,
}

impl CanonicalFormatter {
    /// Create a new `CanonicalFormatter` object.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Convenience method to return the appropriate writer given the current context.
    ///
    /// If we are currently writing an object (that is, if `!self.object_stack.is_empty()`), we
    /// need to write the value to either the next key or next value depending on that state
    /// machine. See the docstrings for `Object` for more detail.
    ///
    /// If we are not currently writing an object, pass through `writer`.
    #[inline]
    fn writer<'a, W>(&'a mut self, writer: &'a mut W) -> impl WriteExt + 'a
    where
        W: io::Write + ?Sized,
    {
        self.object_stack.last_mut().map_or_else(
            || {
                // TODO: This is annoying. Following the migration to the new trait solver, we have to box here to keep the code compiling.
                //
                // It's weird that boxing solves it here since the trait solver still needs to prove that `BufferedWriter<&mut W>` implements `WriteExt` to allow for the coercion to the trait object.
                // So returning the raw unboxed type should also make sense to the trait solver. But apparently it doesn't.
                //
                // How unfortunate. But at least, looking at the benchmark, it doesn't have _that much_ of an impact.
                let boxed = Box::new(BufferedWriter::new(writer)) as Box<dyn WriteExt + 'a>;
                Either::Right(boxed)
            },
            |object| {
                let container = match object.state {
                    Collecting::Key(ref mut key) => key,
                    Collecting::Value { ref mut value, .. } => value,
                };

                Either::Left(container)
            },
        )
    }

    /// Returns a mutable reference to the top of the object stack.
    #[inline]
    fn obj_mut(&mut self) -> io::Result<&mut Object> {
        self.object_stack.last_mut().ok_or_else(|| {
            io::Error::other(
                "Serializer called an object method without calling begin_object first",
            )
        })
    }
}

/// Wraps `sonic_rs::CompactFormatter` to use the appropriate writer (see
/// `CanonicalFormatter::writer`).
macro_rules! wrapper {
    ($f:ident) => {
        #[inline]
        fn $f<W: io::Write + ?Sized>(&mut self, writer: &mut W) -> io::Result<()> {
            CompactFormatter.$f(&mut self.writer(writer))
        }
    };

    ($f:ident, $t:ty) => {
        #[inline]
        fn $f<W: io::Write + ?Sized>(&mut self, writer: &mut W, arg: $t) -> io::Result<()> {
            CompactFormatter.$f(&mut self.writer(writer), arg)
        }
    };

    ($( $f:ident $(, $t:ty)?);* $(;)?) => {
        $(
            wrapper!(
                $f
                $(, $t)?
            );
        )*
    };
}

macro_rules! float_err {
    () => {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "floating point numbers are not allowed",
        ))
    };
}

impl Formatter for CanonicalFormatter {
    wrapper! {
        write_null;
        write_bool, bool;
    }

    wrapper! {
        write_i8, i8;
        write_i16, i16;
        write_i32, i32;
        write_i64, i64;
        write_i128, i128;
    }

    wrapper! {
        write_u8, u8;
        write_u16, u16;
        write_u32, u32;
        write_u64, u64;
        write_u128, u128;
    }

    wrapper! {
        write_byte_array, &[u8];
    }

    wrapper! {
        begin_string;
        end_string;
    }

    wrapper! {
        begin_array;
        end_array;
        begin_array_value, bool;
        end_array_value;
    }

    #[inline]
    fn write_f32<W: io::Write + ?Sized>(&mut self, _writer: &mut W, _value: f32) -> io::Result<()> {
        float_err!()
    }

    #[inline]
    fn write_f64<W: io::Write + ?Sized>(&mut self, _writer: &mut W, _value: f64) -> io::Result<()> {
        float_err!()
    }

    // If sonic_rs's `arbitrary_precision` feature is enabled, all numbers are internally stored as strings,
    // and this method is always used (even for floating point values).
    #[inline]
    fn write_number_str<W: io::Write + ?Sized>(
        &mut self,
        writer: &mut W,
        value: &str,
    ) -> io::Result<()> {
        if memchr3(b'.', b'e', b'E', value.as_bytes()).is_some() {
            float_err!()
        } else {
            CompactFormatter.write_number_str(&mut self.writer(writer), value)
        }
    }

    fn write_string_fast<W>(
        &mut self,
        writer: &mut W,
        value: &str,
        need_quote: bool,
    ) -> io::Result<()>
    where
        W: WriteExt + ?Sized,
    {
        if need_quote {
            self.writer(writer).write_all(b"\"")?;
        }

        let normalizer = const { ComposingNormalizer::new_nfc() };
        for ch in normalizer.normalize_iter(value.chars()) {
            // CJSON wants us to escape backslashes and double quotes.
            // And only backslashes and double quotes.
            if matches!(ch, '\\' | '"') {
                self.writer(writer).write_all(b"\\")?;
            }

            self.writer(writer)
                .write_all(ch.encode_utf8(&mut [0; 4]).as_bytes())?;
        }

        if need_quote {
            self.writer(writer).write_all(b"\"")?;
        }

        Ok(())
    }

    // Here are the object methods. Because keys must be sorted, we serialize the object's keys and
    // values in memory as a `BTreeMap`, then write it all out when `end_object_value` is called.

    #[inline]
    fn begin_object<W: io::Write + ?Sized>(&mut self, writer: &mut W) -> io::Result<()> {
        CompactFormatter.begin_object(&mut self.writer(writer))?;
        self.object_stack.push(Object::default());
        Ok(())
    }

    #[inline]
    fn end_object<W: io::Write + ?Sized>(&mut self, writer: &mut W) -> io::Result<()> {
        let object = self.object_stack.pop().ok_or_else(|| {
            io::Error::other(
                "sonic_rs called Formatter::end_object object method
                 without calling begin_object first",
            )
        })?;

        let mut first = true;
        let mut writer = self.writer(writer);

        for (key, value) in object.obj {
            CompactFormatter.begin_object_key(&mut writer, first)?;
            writer.write_all(&key)?;
            CompactFormatter.end_object_key(&mut writer)?;

            CompactFormatter.begin_object_value(&mut writer)?;
            writer.write_all(&value)?;
            CompactFormatter.end_object_value(&mut writer)?;

            first = false;
        }

        CompactFormatter.end_object(&mut writer)
    }

    #[inline]
    fn begin_object_key<W: io::Write + ?Sized>(
        &mut self,
        _writer: &mut W,
        _first: bool,
    ) -> io::Result<()> {
        let object = self.obj_mut()?;
        object.state = Collecting::Key(Vec::new());

        Ok(())
    }

    #[inline]
    fn end_object_key<W: io::Write + ?Sized>(&mut self, _writer: &mut W) -> io::Result<()> {
        let object = self.obj_mut()?;

        let Collecting::Key(key) = &mut object.state else {
            unreachable!();
        };

        object.state = Collecting::Value {
            key: mem::take(key),
            value: Vec::new(),
        };

        Ok(())
    }

    #[inline]
    fn begin_object_value<W: io::Write + ?Sized>(&mut self, _writer: &mut W) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn end_object_value<W: io::Write + ?Sized>(&mut self, _writer: &mut W) -> io::Result<()> {
        let object = self.obj_mut()?;
        let Collecting::Value { key, value } = &mut object.state else {
            unreachable!();
        };

        object.obj.insert(mem::take(key), mem::take(value));

        Ok(())
    }

    #[inline]
    fn write_raw_value<W: io::Write + ?Sized>(
        &mut self,
        writer: &mut W,
        fragment: &str,
    ) -> io::Result<()> {
        let mut ser = Serializer::with_formatter(self.writer(writer), Self::new());
        sonic_rs::from_str::<sonic_rs::Value>(fragment)?.serialize(&mut ser)?;
        Ok(())
    }
}

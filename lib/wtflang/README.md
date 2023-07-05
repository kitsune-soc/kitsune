# wtflang

Small library for converting from/to ISO-639 codes (two letter, three letter alpha-B and alpha-T), and for outputting their names.

While there are similar existing approaches, such as [rust_iso639](https://docs.rs/rust_iso639), 
most of them don't represent the entries as an enum, therefore not enabling compact storage for things like databases.

This library also handles non-existent codes for certain languages via the `Option` type instead of using empty string slices.

We also have built-in support for converting the three-letter codes from `whichlang`.

## Source

The CSV file used for codegen is sourced from [Datahub](https://datahub.io/core/language-codes) which sources its information from the library of congress.

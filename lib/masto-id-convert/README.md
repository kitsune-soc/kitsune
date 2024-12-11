# masto-id-convert

Convert a Mastodon snowflake ID into a UUID v7 while preserving the timestamp component. Fast.

## How?

The unix timestamp is preserved as-is, the 2-bytes sequence is kept as-is since we expect the sequence to be already unique.

## Performance

Tested inside a NixOS installation on a Ryzen 7 7840U:

```text
Timer precision: 20 ns
process                   fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ process ASCII                        │               │               │               │         │
│  ╰─ 110368129515784116  20.82 ns      │ 192 ns        │ 20.98 ns      │ 24.25 ns      │ 100     │ 12800
╰─ process integer                      │               │               │               │         │
   ╰─ 110368129515784116  14.13 ns      │ 17.18 ns      │ 14.17 ns      │ 14.2 ns       │ 100     │ 25600
```

Processing a single Mastodon snowflake takes ~14ns

## License

`masto-id-convert` is licensed under the [MIT license](http://opensource.org/licenses/MIT).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you,
shall be licensed as above, without any additional terms or conditions.

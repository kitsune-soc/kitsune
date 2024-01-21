# masto-id-convert

Convert a Mastodon snowflake ID into a UUID v7 while preserving the timestamp component. Fast.

## How?

The unix timestamp is preserved as-is, the 2-bytes sequence is stretched out via the WyRand PRNG algorithm.

## Performance

Tested inside a WSL2 installation on an AMD Ryzen 7 7700X:

```text
process integer 110368129515784116
                        time:   [2.9570 ns 2.9604 ns 2.9671 ns]
Found 11 outliers among 100 measurements (11.00%)
  5 (5.00%) high mild
  6 (6.00%) high severe

process ASCII 110368129515784116
                        time:   [10.109 ns 10.114 ns 10.119 ns]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
```

Processing a single Mastodon snowflake takes ~10ns

## License

`masto-id-convert` is licensed under the [MIT license](http://opensource.org/licenses/MIT).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you,
shall be licensed as above, without any additional terms or conditions.

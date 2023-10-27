# masto-id-convert

Convert a Mastodon snowflake ID into a UUID v7 while preserving the timestamp component. Fast.

## How?

The unix timestamp is preserved as-is, the 2-bytes sequence is stretched out via the WyRand PRNG algorithm.

## Performance

Tested inside a WSL2 installation on a Ryzen 5 3600X:

```text
process 110368129515784116
        time:   [16.675 ns 16.822 ns 17.037 ns]
        change: [-1.2226% -0.3915% +0.4911%] (p = 0.37 > 0.05)
```

Processing a single Mastodon snowflake takes ~17ns

## License

`masto-id-convert` is licensed under the [MIT license](http://opensource.org/licenses/MIT).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, 
shall be licensed as above, without any additional terms or conditions.

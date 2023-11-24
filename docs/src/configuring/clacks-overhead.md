# Clacks Overhead

Clacks Overhead is a non-standard HTTP header used for as something like a silent memorial.  
The header is appended to each response via a middleware.

The header looks something like this:

```
X-Clacks-Overhead: GNU [Name 1 here], [Name 2 here]
```

[More info about this header](https://xclacksoverhead.org/home/about)

---

The names for the header can be configured like so:

```toml
[server]
clacks-overhead = [
    "Natalie Nguyen",
    "John \"Soap\" MacTavish"
]
```
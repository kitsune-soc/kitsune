# geomjeungja

Domain verification via TXT records

## About

Geomjeungja is a small library for verifying domain ownership via the user setting a TXT record.  
It is only compatible with Tokio at the moment but this might change in the future.

It ships with one default verification strategy. This strategy is for validating structures looking like this: `[key]=[value]`.  
In case you need anything more complicated, consider implementing your own strategy.

A strategy is an asynchronous fallible operation with its own context that operates over an iterator of string slices that represent the TXT records.

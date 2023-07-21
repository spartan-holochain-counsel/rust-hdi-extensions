[back to README.md](README.md)


# Contributing

## Overview
This package is designed to provide an additional layer of logic on top of the Holochain
[`hdi`](https://docs.rs/hdi) logic.


## Development

### Environment

- Enter `nix develop` for other development environment dependencies.

### Building
This is a library, not a binary.  No build required


### Testing

To run all tests with logging
```
make test-debug
```

- `make test-unit-debug` - **Rust tests only**
- `make test-integration-debug` - **Integration tests only**


> **NOTE:** remove `-debug` to run tests without logging

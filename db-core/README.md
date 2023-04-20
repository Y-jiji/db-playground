# Sanity Checks
```
cargo test check
```

# Inlined Little Benchmarks
```
cargo test bench --profile=release --lib -- --nocapture --test-threads=1
```
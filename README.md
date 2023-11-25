# lid (Lily's ID)

I wanted something that I could trust with being unique enough, while also being extremely fast and light on system resources.

For some of my projects I used a [flakeid](https://github.com/boundary/flake) implementation I [wrote in Rust](https://git.radial.gg/opus/flakeid-rs), but this requires it to be ran as its own service per-machine if it's being shared by multiple services.

By default this uses Base32 and 28 bytes total, This gives approx. 1393796574908163946345982392040522594123776 (32^28) possible IDs.

# Example usage
```rust
use lid::LID;

fn main() {
    // This uses the global Mutex LID instance. It's slightly slower due to the Mutex.
    println!("{}", lid::generate_lid());

    let mut lid = LID::default();
    println!("{}", lid.generate());
}
```

# Benchmarks
(This is of course on my machine, you can test on your own if you'd prefer.)
```
generate_lid() (global) time:   [18.332 ns 18.406 ns 18.502 ns]
oid.generate()          time:   [13.732 ns 13.799 ns 13.877 ns]
flakeid-rs              time:   [31.943 ns 31.981 ns 32.020 ns]
colorid                 time:   [383.17 ns 384.42 ns 386.12 ns]
nanoid                  time:   [723.85 ns 725.15 ns 726.48 ns]
snowflaked              time:   [243.86 ns 244.17 ns 244.50 ns]
```
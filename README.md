# `const_power_of_two`

[<img alt="github" src="https://img.shields.io/badge/github-seancroach%2Fconst__power__of__two-ab9df2?style=for-the-badge&logo=github" height="20">](https://github.com/seancroach/const_power_of_two)
[<img alt="crates.io" src="https://img.shields.io/crates/v/const_power_of_two?style=for-the-badge&logo=rust" height="20">](https://crates.io/crates/const_power_of_two)
[<img alt="docs.rs" src="https://img.shields.io/docsrs/const_power_of_two?style=for-the-badge&logo=docsdotrs" height="20">](https://docs.rs/const_power_of_two)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/seancroach/const_power_of_two/ci.yml?style=for-the-badge&logo=github" height="20">](https://github.com/seancroach/const_power_of_two/actions?query=branch%3Amain)

A crate for working with constant generics that are powers of two.

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
const_power_of_two = "0.1"
```

Then, import the corresponding trait for your argument type, and add it to your
trait or implementation's `where` bounds:

```rust
use const_power_of_two::PowerOfTwoUsize;

struct Test;

trait MyTrait<const ALIGNMENT: usize>
where
    usize: PowerOfTwoUsize<ALIGNMENT>,
{
    // ...
}

// NOTE: This is valid, and no error is emitted.
impl MyTrait<4> for Test {}

// NOTE: This will emit an error at compile-time.
impl MyTrait<10> for Test {}
```

The integer type is what implements the trait, as you can see above. It's not
the most common Rust pattern, but it's easy to work with once you've seen it
in action. At compile-time, if `ALIGNMENT` isn't a power of two, an error will
get emitted.

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](https://github.com/seancroach/const_power_of_two/blob/main/LICENSE-APACHE)
  or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  ([LICENSE-MIT](https://github.com/seancroach/const_power_of_two/blob/main/LICENSE-MIT)
  or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

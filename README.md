# globber

[![Build Status](https://travis-ci.org/CJP10/globber.svg?branch=master)](https://travis-ci.org/CJP10/globber)
[![Docs](https://docs.rs/globber/badge.svg)](https://docs.rs/globber)
[![Crate](https://meritbadge.herokuapp.com/globber)](https://crates.io/crates/globber)

This crate provides matching of strings to extended glob patterns.
Only matching is supported currently and actual filesystem look up is on the road map.

If you need filesystem look up the [glob] crate is amazing and was a major inspiration for this crate.

## Usage
Add the following to your `Cargo.toml`
```toml
[dependencies]
globber = "0.1"
```

## Examples

#### Wildcards
```rust
let pattern = Pattern::new("*.rs").unwrap();
assert!(pattern.matches("hey.rs"));
assert!(!pattern.matches("hey.c"));
assert!(pattern.matches("/src/test.rs"));
assert!(!pattern.matches("/src/test.c"));
```
#### Ranges
```rust
let pattern = Pattern::new("[a-z].rs").unwrap();
assert!(pattern.matches("a.rs"));
assert!(pattern.matches("d.rs"));
assert!(pattern.matches("z.rs"));
assert!(!pattern.matches("A.rs"));
assert!(!pattern.matches("Z.rs"));
assert!(!pattern.matches("0.rs"));
```
#### Patterns
```rust
let pattern = Pattern::new("!([a-z]).rs").unwrap();
assert!(!pattern.matches("a.rs"));
assert!(!pattern.matches("d.rs"));
assert!(!pattern.matches("z.rs"));
assert!(pattern.matches("A.rs"));
assert!(pattern.matches("Z.rs"));
assert!(pattern.matches("0.rs"));
```

## Syntax
#### Basic
```
?           is any character
*           any sqeunece of characters
**          matches zero or more sqeuneces of characters
[abc]       matches one character given in the bracket
[a-z]       matches a character in the range inclusively
[!abc]      does not match one character given in the bracket
[!a-z]      does not match a character in the range inclusively
```
#### Extended
```
?(pattern|pattern|pattern) matches zero or one of the patterns
*(pattern|pattern|pattern) matches zero or more of the patterns
+(pattern|pattern|pattern) matches ine or more of the patterns
@(pattern|pattern|pattern) matches exactly one of the patterns
!(pattern|pattern|pattern) matches none of the patterns
```
A pattern is any valid glob pattern e.g, `!(+(ab|def)*+(.jpg|.gif))`

[glob]: https://github.com/rust-lang-nursery/glob

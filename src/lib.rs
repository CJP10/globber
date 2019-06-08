//#![deny(missing_docs)]

//! This crate provides matching of strings to extended glob patterns.
//! Only matching is supported currently and actual filesystem look up is on the road map.
//!
//! If you need filesystem look up the [glob] crate is amazing and was a major inspiration for this crate.
//!
//! # Usage
//! Add the following to your `Cargo.toml`
//! ```toml
//! [dependencies]
//! globber = "0.1"
//! ```
//!
//! # Examples
//!
//! ### Wildcards
//! ```rust
//! # use globber::Pattern;
//! let pattern = Pattern::new("*.rs").unwrap();
//! assert!(pattern.matches("hey.rs"));
//! assert!(!pattern.matches("hey.c"));
//! assert!(pattern.matches("/src/test.rs"));
//! assert!(!pattern.matches("/src/test.c"));
//! ```
//! ### Ranges
//! ```rust
//! # use globber::Pattern;
//! let pattern = Pattern::new("[a-z].rs").unwrap();
//! assert!(pattern.matches("a.rs"));
//! assert!(pattern.matches("d.rs"));
//! assert!(pattern.matches("z.rs"));
//! assert!(!pattern.matches("A.rs"));
//! assert!(!pattern.matches("Z.rs"));
//! assert!(!pattern.matches("0.rs"));
//! ```
//! ### Patterns
//! ```rust
//! # use globber::Pattern;
//! let pattern = Pattern::new("!([a-z]).rs").unwrap();
//! assert!(!pattern.matches("a.rs"));
//! assert!(!pattern.matches("d.rs"));
//! assert!(!pattern.matches("z.rs"));
//! assert!(pattern.matches("A.rs"));
//! assert!(pattern.matches("Z.rs"));
//! assert!(pattern.matches("0.rs"));
//! ```
//!
//! # Syntax
//! ### Basic
//! ```text
//! ?           is any character
//! *           any sqeunece of characters
//! **          matches zero or more sqeuneces of characters
//! [abc]       matches one character given in the bracket
//! [a-z]       matches a character in the range inclusively
//! [!abc]      does not match one character given in the bracket
//! [!a-z]      does not match a character in the range inclusively
//! ```
//! ### Extended
//! ```text
//! ?(pattern|pattern|pattern) matches zero or one of the patterns
//! *(pattern|pattern|pattern) matches zero or more of the patterns
//! +(pattern|pattern|pattern) matches ine or more of the patterns
//! @(pattern|pattern|pattern) matches exactly one of the patterns
//! !(pattern|pattern|pattern) matches none of the patterns
//! ```
//! A pattern is any valid glob pattern e.g, `!(+(ab|def)*+(.jpg|.gif))`
//!
//! [glob]: https://github.com/rust-lang-nursery/glob

use std::fmt::{Display, Error as FmtError, Formatter};
use std::str::FromStr;

use crate::matcher::Matcher;
use crate::syntax::{Error as SyntaxError, parse};

pub(crate) mod syntax;
pub(crate) mod matcher;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Pattern {
    matcher: Matcher,
}

impl Pattern {
    pub fn new(pattern: &str) -> Result<Self, Error> {
        pattern.parse()
    }

    pub fn matches(&self, input: &str) -> bool {
        self.matcher.matches(input.chars())
    }
}

impl FromStr for Pattern {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            matcher: Matcher::new(
                parse(s)
                    .map_err(|e| Error {
                        inner: e,
                        input: s.to_owned(),
                    })?
            ),
        })
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Error {
    pub(crate) inner: SyntaxError,
    pub(crate) input: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        let mut pos = String::new();
        match self.inner {
            SyntaxError::EmptyPattern(i) | SyntaxError::UnclosedPattern(i)
            | SyntaxError::IllegalChar(i) | SyntaxError::IllegalOr(i)
            | SyntaxError::UnclosedRange(i) | SyntaxError::EmptyRange(i)
            | SyntaxError::IllegalWildcard(i) | SyntaxError::IllegalRecursion(i)
            | SyntaxError::IllegalEscape(i) => {
                for _ in 0..i { pos.push('-') }
                pos.push('^');
            }
        }
        write!(f,
               "Glob syntax error\n-------------------------------------\n{}\n{}\n{}\n-------------------------------------",
               self.inner.desc(), self.input, pos
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Pattern;
    use super::syntax::Error;

    #[test]
    fn wildcards() {
        assert!(Pattern::new("*").unwrap().matches("a"));
        assert!(Pattern::new("**").unwrap().matches("/a/b/c/d/e/f"));
        assert!(Pattern::new("star\\*").unwrap().matches("star*"));
        assert!(Pattern::new("star\\**").unwrap().matches("star*light"));
        assert!(Pattern::new("/var/log/**").unwrap().matches("/var/log/test"));
        assert!(Pattern::new("/var/log/**").unwrap().matches("/var/log/a/b"));
        assert!(Pattern::new("/var/log/**").unwrap().matches("/var/log/a"));
        assert!(Pattern::new("/var/log/**").unwrap().matches("/var/log/a.b"));
        assert!(Pattern::new("a*b").unwrap().matches("a_b"));
        assert!(Pattern::new("a*b*c").unwrap().matches("abc"));
        assert!(!Pattern::new("a*b*c").unwrap().matches("abcd"));
        assert!(Pattern::new("a*b*c").unwrap().matches("a_b_c"));
        assert!(Pattern::new("a*b*c").unwrap().matches("a___b___c"));
        assert!(Pattern::new("abc*abc*abc").unwrap().matches("abcabcabcabcabcabcabc"));
        assert!(!Pattern::new("abc*abc*abc").unwrap().matches("abcabcabcabcabcabcabca"));
        assert!(Pattern::new("a*a*a*a*a*a*a*a*a").unwrap().matches("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
        assert!(Pattern::new("a*b[xyz]c*d").unwrap().matches("abxcdbxcddd"));
        assert!(Pattern::new("/**/[xyz]").unwrap().matches("/a/b/c/x"));
        assert!(Pattern::new("/**/[xyz]").unwrap().matches("/y"));
        assert!(Pattern::new("/**/[xyz]").unwrap().matches("/a/z"));
        assert!(Pattern::new("*.log").unwrap().matches("sys.log"));
        assert!(Pattern::new("sys.*").unwrap().matches("sys.log"));

        let p = Pattern::new("some/**/needle.txt").unwrap();
        assert!(p.matches("some/needle.txt"));
        assert!(p.matches("some/one/needle.txt"));
        assert!(p.matches("some/one/two/needle.txt"));
        assert!(p.matches("some/other/needle.txt"));
        assert!(!p.matches("some/other/notthis.txt"));

        let p = Pattern::new("**").unwrap();
        assert!(p.matches("abcde"));
        assert!(p.matches(""));
        assert!(p.matches(".asdf"));
        assert!(p.matches("/x/.asdf"));

        let p = Pattern::new("some/**/**/needle.txt").unwrap();
        assert!(p.matches("some/needle.txt"));
        assert!(p.matches("some/one/needle.txt"));
        assert!(p.matches("some/one/two/needle.txt"));
        assert!(p.matches("some/other/needle.txt"));
        assert!(!p.matches("some/other/notthis.txt"));

        let p = Pattern::new("**/test").unwrap();
        assert!(p.matches("one/two/test"));
        assert!(p.matches("one/test"));
        assert!(p.matches("test"));

        let p = Pattern::new("/**/test").unwrap();
        assert!(p.matches("/one/two/test"));
        assert!(p.matches("/one/test"));
        assert!(p.matches("/test"));
        assert!(!p.matches("/one/notthis"));
        assert!(!p.matches("/notthis"));

        let p = Pattern::new("**/.*").unwrap();
        assert!(p.matches(".abc"));
        assert!(p.matches("abc/.abc"));
        assert!(!p.matches("ab.c"));
        assert!(!p.matches("abc/ab.c"));
    }

    #[test]
    fn wildcard_errors() {
        assert_eq!(Pattern::new("a/**b").unwrap_err().inner, Error::IllegalRecursion(4));
        assert_eq!(Pattern::new("a/bc**").unwrap_err().inner, Error::IllegalRecursion(3));
        assert_eq!(Pattern::new("a/*****").unwrap_err().inner, Error::IllegalWildcard(4));
        assert_eq!(Pattern::new("a/b**c**d").unwrap_err().inner, Error::IllegalRecursion(2));
        assert_eq!(Pattern::new("a**b").unwrap_err().inner, Error::IllegalRecursion(0));
        assert_eq!(Pattern::new("***").unwrap_err().inner, Error::IllegalWildcard(2));
        assert_eq!(Pattern::new("****").unwrap_err().inner, Error::IllegalWildcard(2));
        assert_eq!(Pattern::new("a**/b").unwrap_err().inner, Error::IllegalRecursion(0));
        assert_eq!(Pattern::new("a/\\***").unwrap_err().inner, Error::IllegalRecursion(3));
    }

    #[test]
    fn ranges() {
        let p = Pattern::new("a[a-z]c").unwrap();
        assert!(p.matches("aac"));
        assert!(p.matches("aec"));
        assert!(p.matches("azc"));
        assert!(!p.matches("a0c"));
        assert!(!p.matches("aAc"));
        assert!(!p.matches("aBc"));
        assert!(!p.matches("aZc"));
    }

    #[test]
    fn range_errors() {
        assert_eq!(Pattern::new("[!]").unwrap_err().inner, Error::EmptyRange(0));
        assert_eq!(Pattern::new("[]").unwrap_err().inner, Error::EmptyRange(0));
        assert_eq!(Pattern::new("[]]]]]").unwrap_err().inner, Error::EmptyRange(0));
        assert_eq!(Pattern::new("[dfsfsdfsdf").unwrap_err().inner, Error::UnclosedRange(10));
        assert_eq!(Pattern::new("[!sdfdsfdf").unwrap_err().inner, Error::UnclosedRange(9));
        assert_eq!(Pattern::new("abc[def").unwrap_err().inner, Error::UnclosedRange(6));
        assert_eq!(Pattern::new("abc[!def").unwrap_err().inner, Error::UnclosedRange(7));
        assert_eq!(Pattern::new("abc[").unwrap_err().inner, Error::EmptyRange(3));
        assert_eq!(Pattern::new("abc[!").unwrap_err().inner, Error::UnclosedRange(4));
        assert_eq!(Pattern::new("abc[d").unwrap_err().inner, Error::UnclosedRange(4));
        assert_eq!(Pattern::new("abc[!d").unwrap_err().inner, Error::UnclosedRange(5));
        assert_eq!(Pattern::new("abc[]").unwrap_err().inner, Error::EmptyRange(3));
        assert_eq!(Pattern::new("abc[!]").unwrap_err().inner, Error::EmptyRange(3));
        assert_eq!(Pattern::new("abc[!]").unwrap_err().inner, Error::EmptyRange(3));
        assert_eq!(Pattern::new("[adc(]").unwrap_err().inner, Error::IllegalChar(4));
        assert_eq!(Pattern::new("[adc[]").unwrap_err().inner, Error::IllegalChar(4));
        assert_eq!(Pattern::new("[adc]]").unwrap_err().inner, Error::IllegalChar(5));
        assert_eq!(Pattern::new("[adc)]").unwrap_err().inner, Error::IllegalChar(4));
    }

    #[test]
    fn pattern_matches() {
        let txt_pat = Pattern::new("*hello.txt").unwrap();
        assert!(txt_pat.matches("hello.txt"));
        assert!(txt_pat.matches("gareth_says_hello.txt"));
        assert!(txt_pat.matches("some/path/to/hello.txt"));
        assert!(txt_pat.matches("some\\path\\to\\hello.txt"));
        assert!(txt_pat.matches("/an/absolute/path/to/hello.txt"));
        assert!(!txt_pat.matches("hello.txt-and-then-some"));
        assert!(!txt_pat.matches("goodbye.txt"));

        let dir_pat = Pattern::new("*some/path/to/hello.txt").unwrap();
        assert!(dir_pat.matches("some/path/to/hello.txt"));
        assert!(dir_pat.matches("a/bigger/some/path/to/hello.txt"));
        assert!(!dir_pat.matches("some/path/to/hello.txt-and-then-some"));
        assert!(!dir_pat.matches("some/other/path/to/hello.txt"));
    }

    #[test]
    fn ranges_plus() {
        let pat = Pattern::new("a[0-9]b").unwrap();
        for i in 0..10 {
            assert!(pat.matches(&format!("a{}b", i)));
        }
        assert!(!pat.matches("a_b"));

        let pat = Pattern::new("a[!0-9]b").unwrap();
        for i in 0..10 {
            assert!(!pat.matches(&format!("a{}b", i)));
        }
        assert!(pat.matches("a_b"));

        let pats = ["[a-z123]", "[1a-z23]", "[123a-z]"];
        for &p in pats.iter() {
            let pat = Pattern::new(p).unwrap();
            for c in "abcdefghijklmnopqrstuvwxyz".chars() {
                assert!(pat.matches(&c.to_string()));
            }
            assert!(pat.matches("1"));
            assert!(pat.matches("2"));
            assert!(pat.matches("3"));
        }

        let pats = ["[abc-]", "[-abc]", "[a-c-]"];
        for &p in pats.iter() {
            let pat = Pattern::new(p).unwrap();
            assert!(pat.matches("a"));
            assert!(pat.matches("b"));
            assert!(pat.matches("c"));
            assert!(pat.matches("-"));
            assert!(!pat.matches("d"));
        }

        let pat = Pattern::new("[2-1]").unwrap();
        assert!(!pat.matches("1"));
        assert!(!pat.matches("2"));

        assert!(Pattern::new("[-]").unwrap().matches("-"));
        assert!(!Pattern::new("[!-]").unwrap().matches("-"));
    }

    #[test]
    fn zero_or_one() {
        let p = Pattern::new("src/?([a-z]|[a-c]).rs").unwrap();
        assert!(!p.matches("src/a.rs"));
        assert!(!p.matches("src/b.rs"));
        assert!(!p.matches("src/c.rs"));
        assert!(p.matches("src/d.rs"));
        assert!(p.matches("src/e.rs"));
        assert!(p.matches("src/f.rs"));
        assert!(!p.matches("src/ggggggggg.rs"));
        assert!(!p.matches("src/0.rs"));
        assert!(!p.matches("src/123456789.rs"));
        assert!(p.matches("src/.rs"));
    }

    #[test]
    fn zero_or_more() {
        let p = Pattern::new("src/*([a-z]|[a-c]).rs").unwrap();
        assert!(p.matches("src/a.rs"));
        assert!(p.matches("src/b.rs"));
        assert!(p.matches("src/c.rs"));
        assert!(p.matches("src/d.rs"));
        assert!(p.matches("src/e.rs"));
        assert!(p.matches("src/f.rs"));
        assert!(!p.matches("src/ggggggggg.rs"));
        assert!(!p.matches("src/0.rs"));
        assert!(!p.matches("src/123456789.rs"));
        assert!(p.matches("src/.rs"));
    }

    #[test]
    fn one_or_more() {
        let p = Pattern::new("src/+([a-z]|[a-c]).rs").unwrap();
        assert!(p.matches("src/a.rs"));
        assert!(p.matches("src/b.rs"));
        assert!(p.matches("src/c.rs"));
        assert!(p.matches("src/d.rs"));
        assert!(p.matches("src/e.rs"));
        assert!(p.matches("src/f.rs"));
        assert!(!p.matches("src/ggggggggg.rs"));
        assert!(!p.matches("src/0.rs"));
        assert!(!p.matches("src/123456789.rs"));
        assert!(!p.matches("src/.rs"));
    }

    #[test]
    fn exactly_one() {
        let p = Pattern::new("src/@([a-z]|[a-c]).rs").unwrap();
        assert!(!p.matches("src/a.rs"));
        assert!(!p.matches("src/b.rs"));
        assert!(!p.matches("src/c.rs"));
        assert!(p.matches("src/d.rs"));
        assert!(p.matches("src/e.rs"));
        assert!(p.matches("src/f.rs"));
        assert!(!p.matches("src/ggggggggg.rs"));
        assert!(!p.matches("src/0.rs"));
        assert!(!p.matches("src/123456789.rs"));
        assert!(!p.matches("src/.rs"));
    }

    #[test]
    fn none_of() {
        let p = Pattern::new("src/!([a-z]|[a-c]).rs").unwrap();
        assert!(!p.matches("src/a.rs"));
        assert!(!p.matches("src/b.rs"));
        assert!(!p.matches("src/c.rs"));
        assert!(!p.matches("src/d.rs"));
        assert!(!p.matches("src/e.rs"));
        assert!(!p.matches("src/f.rs"));
        assert!(p.matches("src/ggggggggg.rs"));
        assert!(!p.matches("src/ggggggggg"));
        assert!(p.matches("src/0.rs"));
        assert!(p.matches("src/123456789.rs"));
        assert!(!p.matches("src/.rs"));
    }

    #[test]
    fn extra() {
        let p = Pattern::new("/var/log/!(containers)*/**").unwrap();
        assert!(p.matches("/var/log/cpus/"));
        assert!(p.matches("/var/log/cpus/core_0.log"));
        assert!(!p.matches("/var/log/containers/"));
        assert!(!p.matches("/var/log/containers/0.log"));

        let p = Pattern::new("!(+(ab|def)*+(.jpg|.gif))").unwrap();
        assert!(!p.matches("ab.jpg"));
        assert!(!p.matches("abc.jpg"));
        assert!(!p.matches("def.jpg"));
        assert!(!p.matches("defgh.jpg"));
        assert!(!p.matches("ab.gif"));
        assert!(!p.matches("abc.gif"));
        assert!(!p.matches("def.gif"));
        assert!(!p.matches("defgh.gif"));
        assert!(p.matches("ced.gif"));
        assert!(p.matches("ced.jpg"));
        assert!(p.matches("test.rs"));
        assert!(p.matches("ab.rs"));

        let p = Pattern::new("!(!(!(!(vec|test)))).rs").unwrap();
        assert!(p.matches("vec.rs"));
        assert!(p.matches("test.rs"));
        assert!(!p.matches("dot.rs"));
        assert!(!p.matches(".rs"));
        assert!(!p.matches("asfsdf.rs"));

        let p = Pattern::new("!(!(!(vec|test))).rs").unwrap();
        assert!(!p.matches("vec.rs"));
        assert!(!p.matches("test.rs"));
        assert!(p.matches("dot.rs"));
        assert!(!p.matches(".rs"));
        assert!(p.matches("asfsdf.rs"));
    }
}
use std::str::FromStr;

use crate::matcher::Matcher;
use crate::syntax::{Error, parse, Token};

pub mod syntax;
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
            matcher: Matcher::new(parse(s)?),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::{CharSpecifier, Error, parse, Token};

    use super::Pattern;

    fn chars(input: &str) -> Vec<Token> {
        input.chars().map(|c| Token::Char(c)).collect()
    }

    fn specifiers(input: &str) -> Vec<CharSpecifier> {
        input.chars().map(|c| CharSpecifier::Char(c)).collect()
    }

    fn range(c1: char, c2: char) -> Vec<CharSpecifier> {
        vec![CharSpecifier::Range(c1, c2)]
    }

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
        assert_eq!(Pattern::new("a/**b").unwrap_err(), Error::IllegalRecursion(4));
        assert_eq!(Pattern::new("a/bc**").unwrap_err(), Error::IllegalRecursion(3));
        assert_eq!(Pattern::new("a/*****").unwrap_err(), Error::IllegalWildcard(4));
        assert_eq!(Pattern::new("a/b**c**d").unwrap_err(), Error::IllegalRecursion(2));
        assert_eq!(Pattern::new("a**b").unwrap_err(), Error::IllegalRecursion(0));
        assert_eq!(Pattern::new("***").unwrap_err(), Error::IllegalWildcard(2));
        assert_eq!(Pattern::new("****").unwrap_err(), Error::IllegalWildcard(2));
        assert_eq!(Pattern::new("a**/b").unwrap_err(), Error::IllegalRecursion(0));
        assert_eq!(Pattern::new("a/\\***").unwrap_err(), Error::IllegalRecursion(3));
    }

    #[test]
    fn ranges() {
        assert_eq!(parse("[a]").unwrap(), vec![Token::AnyOf(specifiers("a"))]);
        assert_eq!(parse("[!a]").unwrap(), vec![Token::NotAnyOf(specifiers("a"))]);
        assert_eq!(parse("[abcdef]").unwrap(), vec![Token::AnyOf(specifiers("abcdef"))]);
        assert_eq!(parse("[!abcdef]").unwrap(), vec![Token::NotAnyOf(specifiers("abcdef"))]);
        assert_eq!(parse("[a-z]").unwrap(), vec![Token::AnyOf(range('a', 'z'))]);
        assert_eq!(parse("[!a-z]").unwrap(), vec![Token::NotAnyOf(range('a', 'z'))]);

        let mut tokens = chars("abc");
        tokens.push(Token::NotAnyOf(range('a', 'z')));
        assert_eq!(parse("abc[!a-z]").unwrap(), tokens);

        let mut tokens = chars("abc");
        tokens.push(Token::AnyOf(range('a', 'z')));
        assert_eq!(parse("abc[a-z]").unwrap(), tokens);
    }

    #[test]
    fn range_errors() {
        assert_eq!(Pattern::new("[!]").unwrap_err(), Error::EmptyRange(0));
        assert_eq!(Pattern::new("[]").unwrap_err(), Error::EmptyRange(0));
        assert_eq!(Pattern::new("[]]]]]").unwrap_err(), Error::EmptyRange(0));
        assert_eq!(Pattern::new("[dfsfsdfsdf").unwrap_err(), Error::IllegalRange(0));
        assert_eq!(Pattern::new("[!sdfdsfdf").unwrap_err(), Error::IllegalRange(0));
        assert_eq!(Pattern::new("abc[def").unwrap_err(), Error::IllegalRange(3));
        assert_eq!(Pattern::new("abc[!def").unwrap_err(), Error::IllegalRange(3));
        assert_eq!(Pattern::new("abc[").unwrap_err(), Error::IllegalRange(3));
        assert_eq!(Pattern::new("abc[!").unwrap_err(), Error::IllegalRange(3));
        assert_eq!(Pattern::new("abc[d").unwrap_err(), Error::IllegalRange(3));
        assert_eq!(Pattern::new("abc[!d").unwrap_err(), Error::IllegalRange(3));
        assert_eq!(Pattern::new("abc[]").unwrap_err(), Error::EmptyRange(3));
        assert_eq!(Pattern::new("abc[!]").unwrap_err(), Error::EmptyRange(3));
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
}
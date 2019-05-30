use std::str::FromStr;

use crate::syntax::{Error, parse, Token};
use crate::matcher::Matcher;

pub mod syntax;
pub(crate) mod matcher;

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
    use super::Pattern;
    use crate::syntax::{CharSpecifier, Error, parse, Token};

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
    fn test_wildcard() {
        assert_eq!(parse("*").unwrap(), vec![Token::AnySequence]);
        assert_eq!(parse("**").unwrap(), vec![Token::AnyRecursive]);
        assert_eq!(parse("\\*").unwrap(), vec![Token::Char('*')]);
        assert_eq!(parse("\\**").unwrap(), vec![Token::Char('*'), Token::AnySequence]);

        let mut tokens = chars("/va");
        tokens.push(Token::AnySequence);
        tokens.append(&mut chars("r"));
        tokens.push(Token::AnySequence);
        tokens.append(&mut chars("/log"));
        tokens.push(Token::AnySequence);
        tokens.append(&mut chars("/"));
        tokens.push(Token::AnyRecursive);
        assert_eq!(parse("/va*r*/log*/**").unwrap(), tokens);
    }

    #[test]
    fn test_wildcard_errors() {
        assert_eq!(parse("a/**b").unwrap_err(), Error::IllegalRecursion(4));
        assert_eq!(parse("a/bc**").unwrap_err(), Error::IllegalRecursion(3));
        assert_eq!(parse("a/*****").unwrap_err(), Error::IllegalWildcard(4));
        assert_eq!(parse("a/b**c**d").unwrap_err(), Error::IllegalRecursion(2));
        assert_eq!(parse("a**b").unwrap_err(), Error::IllegalRecursion(0));
        assert_eq!(parse("***").unwrap_err(), Error::IllegalWildcard(2));
        assert_eq!(parse("****").unwrap_err(), Error::IllegalWildcard(2));
        assert_eq!(parse("a**/b").unwrap_err(), Error::IllegalRecursion(0));
        assert_eq!(parse("a/\\***").unwrap_err(), Error::IllegalRecursion(3));
    }

    #[test]
    fn test_ranges() {
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
    fn test_range_errors() {
        assert_eq!(parse("[!]").unwrap_err(), Error::EmptyRange(0));
        assert_eq!(parse("[]").unwrap_err(), Error::EmptyRange(0));
        assert_eq!(parse("[]]]]]").unwrap_err(), Error::EmptyRange(0));
        assert_eq!(parse("[dfsfsdfsdf").unwrap_err(), Error::IllegalRange(0));
        assert_eq!(parse("[!sdfdsfdf").unwrap_err(), Error::IllegalRange(0));
    }

    #[test]
    fn test_wildcards() {
        assert!(Pattern::new("a*b").unwrap().matches("a_b"));
        assert!(Pattern::new("a*b*c").unwrap().matches("abc"));
        assert!(!Pattern::new("a*b*c").unwrap().matches("abcd"));
        assert!(Pattern::new("a*b*c").unwrap().matches("a_b_c"));
        assert!(Pattern::new("a*b*c").unwrap().matches("a___b___c"));
        assert!(Pattern::new("abc*abc*abc")
            .unwrap()
            .matches("abcabcabcabcabcabcabc"));
        assert!(!Pattern::new("abc*abc*abc")
            .unwrap()
            .matches("abcabcabcabcabcabcabca"));
        assert!(Pattern::new("a*a*a*a*a*a*a*a*a")
            .unwrap()
            .matches("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
//        assert!(Pattern::new("a*b[xyz]c*d").unwrap().matches("abxcdbxcddd"));
    }
}
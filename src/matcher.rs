use std::path::is_separator;
use std::str::Chars;

use Status::*;

use crate::syntax::{CharSpecifier, Token};

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub(crate) struct Matcher {
    tokens: Vec<Token>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Status {
    Match,
    Retryable,
    NoMatch,
}

impl Matcher {
    pub(crate) fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
        }
    }

    pub(crate) fn matches(&self, input: Chars) -> bool {
        self.match_index(0, input) == Status::Match
    }

    fn match_index(&self, i: usize, mut input: Chars) -> Status {
        for (ti, token) in self.tokens[i..].iter().enumerate() {
            match token {
                Token::AnyRecursive | Token::AnySequence => {
                    let result = self.match_index(i + ti + 1, input.clone());
                    match result {
                        Status::Retryable => {}
                        _ => return result,
                    }

                    if let Some(t) = self.tokens.get(i + ti + 1) {
                        match t {
                            Token::Char(c) if is_separator(*c) => {
                                match self.match_index(i + ti + 2, input.clone()) {
                                    Status::Retryable => {}
                                    m => return m,
                                }
                            }
                            _ => {}
                        }
                    }

                    while let Some(_) = input.next() {
                        match self.match_index(i + ti + 1, input.clone()) {
                            Status::Retryable => {}
                            m => return m,
                        }
                    }
                }
                Token::Char(c) => {
                    let next = match input.next() {
                        Some(c) => c,
                        None => return Status::NoMatch,
                    };

                    if *c != next {
                        return Status::Retryable;
                    }
                }
                Token::AnyChar => { return Status::Match; }
                Token::AnyOf(specifiers) => {
                    let next = match input.next() {
                        Some(c) => c,
                        None => return Status::NoMatch,
                    };

                    return match_specifiers(specifiers, next);
                }
                Token::NotAnyOf(specifiers) => {
                    let next = match input.next() {
                        Some(c) => c,
                        None => return Status::NoMatch,
                    };

                    return match match_specifiers(specifiers, next) {
                        Retryable => Match,
                        Match => Retryable,
                        _ => { unreachable!() }
                    };
                }
                _ => { unreachable!() }
            }
        }

        match input.next() {
            Some(_) => { Status::Retryable }
            None => { Status::Match }
        }
    }
}

fn match_specifiers(specifiers: &Vec<CharSpecifier>, c: char) -> Status {
    for specifier in specifiers {
        match specifier {
            CharSpecifier::Char(c1) => {
                if c == *c1 {
                    return Match;
                }
            }
            CharSpecifier::Range(start, end) => {
                if c >= *start && c <= *end {
                    return Match;
                }
            }
        }
    }
    Retryable
}
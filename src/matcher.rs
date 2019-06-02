use std::path::is_separator;
use std::str::Chars;

use crate::matcher::Status::*;
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
        match_index(&self.tokens, 0, input) == Status::Match
    }
}

fn match_index(tokens: &Vec<Token>, i: usize, mut input: Chars) -> Status {
    for (ti, token) in tokens[i..].iter().enumerate() {
        match token {
            Token::AnyRecursive | Token::AnySequence => {
                let result = match_index(tokens, i + ti + 1, input.clone());
                match result {
                    Status::Retryable => {}
                    _ => return result,
                }

                if *token == Token::AnyRecursive {
                    if let Some(t) = tokens.get(i + ti + 1) {
                        match t {
                            Token::Char(c) if is_separator(*c) => {
                                match match_index(tokens, i + ti + 2, input.clone()) {
                                    Status::Retryable => {}
                                    m => return m,
                                }
                            }
                            _ => {}
                        }
                    }
                }

                while let Some(_) = input.next() {
                    match match_index(tokens, i + ti + 1, input.clone()) {
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
                    None => return NoMatch,
                };

                match match_specifiers(specifiers, next) {
                    Match => {}
                    Retryable => return Retryable,
                    _ => { unreachable!() }
                }
            }
            Token::NotAnyOf(specifiers) => {
                let next = match input.next() {
                    Some(c) => c,
                    None => return NoMatch,
                };

                match match_specifiers(specifiers, next) {
                    Retryable => {}
                    Match => return Retryable,
                    _ => { unreachable!() }
                };
            }
            Token::ZeroOrOne(patterns) => {
                let mut matches = 0;

                for t in patterns {
                    let mut t = t.clone();
                    t.extend_from_slice(&tokens[i + ti + 1..]);

                    match match_index(&mut t, 0, input.clone()) {
                        Match => matches += 1,
                        _ => {}
                    }

                    if matches > 1 {
                        return Retryable;
                    }
                }

                if matches == 1 {
                    return Match;
                }

                return match_index(tokens, i + ti + 1, input);
            }
            Token::ZeroOrMore(patterns) => {
                for t in patterns {
                    let mut t = t.clone();
                    t.extend_from_slice(&tokens[i + ti + 1..]);

                    match match_index(&mut t, 0, input.clone()) {
                        Match => return Match,
                        _ => {}
                    }
                }

                return match_index(tokens, i + ti + 1, input);
            }
            Token::OneOrMore(patterns) => {
                for t in patterns {
                    let mut t = t.clone();
                    t.extend_from_slice(&tokens[i + ti + 1..]);

                    match match_index(&mut t, 0, input.clone()) {
                        Match => return Match,
                        _ => {}
                    }
                }

                return Retryable;
            }
            Token::ExactlyOne(patterns) => {
                let mut matches = 0;

                for t in patterns {
                    let mut t = t.clone();
                    t.extend_from_slice(&tokens[i + ti + 1..]);

                    match match_index(&mut t, 0, input.clone()) {
                        Match => matches += 1,
                        _ => {}
                    }

                    if matches > 1 {
                        return Retryable;
                    }
                }

                if matches == 1 {
                    return Match;
                }

                return Retryable;
            }
            Token::NoneOf(patterns) => {
                for t in patterns {
                    let mut t = t.clone();
                    t.extend_from_slice(&tokens[i + ti + 1..]);

                    match match_index(&mut t, 0, input.clone()) {
                        Match => return Retryable,
                        _ => {}
                    }
                }

                while let Some(_) = input.next() {
                    match match_index(tokens, i + ti + 1, input.clone()) {
                        Status::Retryable => {}
                        m => return m,
                    }
                }
            }
        }
    }

    match input.next() {
        Some(_) => { Status::Retryable }
        None => { Status::Match }
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
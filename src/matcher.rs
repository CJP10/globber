use std::path::is_separator;
use std::str::Chars;

use crate::syntax::Token;

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

                    while let Some(c) = input.next() {
                        let is_separator = is_separator(c);

                        match *token {
                            Token::AnyRecursive => {},
                            Token::AnySequence if is_separator => { return Status::Retryable; }
                            Token::AnySequence => {}
                            _ => {unreachable!()}
                        };

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
                _ => { unreachable!() }
            }
        }

        match input.next() {
            Some(_) => { Status::Retryable }
            None => { Status::Match }
        }
    }
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum Token {
    // ?
    AnyChar,
    // *
    AnySequence,
    // **
    AnyRecursive,
    // [abc-z123]
    AnyOf(Vec<CharSpecifier>),
    // [!abc-z123]
    NotAnyOf(Vec<CharSpecifier>),
    // v a r l o g
    Char(char),
    // ?(pattern|pattern|pattern)
    ZeroOrOne(Vec<Vec<Token>>),
    // *(pattern|pattern|pattern)
    ZeroOrMore(Vec<Vec<Token>>),
    // +(pattern|pattern|pattern)
    OneOrMore(Vec<Vec<Token>>),
    // @(pattern|pattern|pattern)
    ExactlyOne(Vec<Vec<Token>>),
    // !(pattern|pattern|pattern)
    NoneOf(Vec<Vec<Token>>),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum CharSpecifier {
    Char(char),
    Range(char, char),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Error {
    IllegalPattern(usize),
    IllegalOr(usize),
    IllegalRange(usize),
    EmptyRange(usize),
    // only ** and * are allowed
    IllegalWildcard(usize),
    // only ** and * are allowed
    IllegalRecursion(usize),
    // when a \ is not followed by a char
    IllegalEscape(usize),
}

pub(crate) fn parse(input: &str) -> Result<Vec<Token>, Error> {
    Parser::new(input).parse()
}

struct Parser {
    chars: Vec<char>,
    i: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            i: 0,
        }
    }

    fn parse(&mut self) -> Result<Vec<Token>, Error> {
        let mut tokens = Vec::new();

        while self.i < self.chars.len() {
            if self.i + 1 < self.chars.len() {
                let token = match (self.chars[self.i], self.chars[self.i + 1]) {
                    ('?', '(') => Some(Token::ZeroOrOne(self.parse_patterns()?)),
                    ('*', '(') => Some(Token::ZeroOrMore(self.parse_patterns()?)),
                    ('+', '(') => Some(Token::OneOrMore(self.parse_patterns()?)),
                    ('@', '(') => Some(Token::ExactlyOne(self.parse_patterns()?)),
                    ('!', '(') => Some(Token::NoneOf(self.parse_patterns()?)),
                    _ => None,
                };

                if let Some(t) = token {
                    tokens.push(t);
                    continue;
                }
            }

            let token = match self.chars[self.i] {
                '?' => {
                    self.i += 1;
                    Token::AnyChar
                }
                '*' => self.parse_wildcards()?,
                '\\' => self.parse_escape()?,
                '[' => self.parse_range()?,
                c => {
                    self.i += 1;
                    Token::Char(c)
                }
            };

            tokens.push(token);
        }

        Ok(tokens)
    }

    fn parse_wildcards(&mut self) -> Result<Token, Error> {
        let mut token = Token::AnySequence;
        let start = self.i;
        let next = self.i + 1;

        // check if the next char is a *, if so we found **
        if next < self.chars.len() && self.chars[next] == '*' {
            token = Token::AnyRecursive;

            // check that to the left of the first * is either no char or a /
            if start > 0 && self.chars[start - 1] != '/' {
                return Err(Error::IllegalRecursion(start - 1));
            }

            // check that to the right of the last * is either no char or a /
            if next + 1 < self.chars.len() {
                match self.chars[next + 1] {
                    '/' => {}
                    '*' => { return Err(Error::IllegalWildcard(next + 1)); }
                    _ => { return Err(Error::IllegalRecursion(next + 1)); }
                }
            }

            self.i = next + 1;
            Ok(token)
        } else {
            self.i = next;
            Ok(token)
        }
    }

    fn parse_escape(&mut self) -> Result<Token, Error> {
        if self.i + 1 >= self.chars.len() {
            return Err(Error::IllegalEscape(self.i));
        }

        self.i += 2;
        Ok(Token::Char(self.chars[self.i - 1]))
    }

    fn parse_range(&mut self) -> Result<Token, Error> {
        let start = self.i;
        let mut first_char = self.i + 1;
        let end;

        if first_char >= self.chars.len() {
            return Err(Error::IllegalRange(start));
        }

        let negated = match self.chars[first_char] {
            '!' => {
                first_char += 1;
                true
            }
            ']' => { return Err(Error::EmptyRange(start)); }
            _ => false,
        };

        let chars = match self.chars[first_char..].iter().position(|x| *x == ']') {
            Some(j) => {
                end = first_char + j;
                parse_char_specifiers(&self.chars[first_char..first_char + j])
            }
            None => { return Err(Error::IllegalRange(start)); }
        };

        if chars.is_empty() {
            return Err(Error::EmptyRange(start));
        }

        self.i = end + 1;

        if negated {
            Ok(Token::NotAnyOf(chars))
        } else {
            Ok(Token::AnyOf(chars))
        }
    }

    fn parse_patterns(&mut self) -> Result<Vec<Vec<Token>>, Error> {
        let first_char = self.i + 2;
        let mut brace_level = 0;
        let mut brace_sequence = None;

        for (i, c) in self.chars[first_char..].iter().enumerate() {
            match *c {
                '(' => brace_level += 1,
                ')' if brace_level == 0 => {
                    brace_sequence = Some(&self.chars[first_char..first_char + i]);
                    break;
                }
                ')' => brace_level -= 1,
                _ => {}
            }
        }

        let brace_sequence = match brace_sequence {
            Some(s) => s,
            None => { return Err(Error::IllegalPattern(first_char)); }
        };

        let mut patterns = Vec::new();
        let mut brace_level = 0;
        let mut last_pattern = 0;
        for (i, c) in brace_sequence.iter().enumerate() {
            match *c {
                '(' => brace_level += 1,
                ')' => brace_level -= 1,
                '|' if brace_level == 0 => {
                    let chars = &self.chars[first_char + last_pattern..first_char + i];
                    if chars.len() == 0 {
                        return Err(Error::IllegalOr(first_char));
                    }
                    last_pattern = i + 1;
                    patterns.push(chars)
                }
                _ => {}
            }
        }

        let chars = &self.chars[first_char + last_pattern..first_char + brace_sequence.len()];
        if chars.len() == 0 {
            return Err(Error::IllegalOr(first_char));
        }
        patterns.push(chars);

        let mut tokens = Vec::new();
        for p in patterns {
            let mut pattern = String::new();
            for c in p {
                pattern.push(*c);
            }
            tokens.push(parse(&pattern)?)
        }

        self.i = first_char + brace_sequence.len() + 1;
        Ok(tokens)
    }
}

fn parse_char_specifiers(s: &[char]) -> Vec<CharSpecifier> {
    let mut cs = Vec::new();
    let mut i = 0;
    while i < s.len() {
        if i + 3 <= s.len() && s[i + 1] == '-' {
            cs.push(CharSpecifier::Range(s[i], s[i + 2]));
            i += 3;
        } else {
            cs.push(CharSpecifier::Char(s[i]));
            i += 1;
        }
    }
    cs
}
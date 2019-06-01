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
    EmptyPattern(usize),
    UnclosedPattern(usize),
    IllegalChar(usize),
    IllegalOr(usize),
    IllegalRange(usize),
    UnclosedRange(usize),
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
                ']' | '(' | ')' | '|' => { return Err(Error::IllegalChar(self.i)); }
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

        let mut escaped = false;
        let mut chars = None;
        for (i, c) in self.chars[first_char..].iter().enumerate() {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                ']' => {
                    chars = Some(&self.chars[first_char..first_char + i]);
                    break;
                }
                '[' | '(' | ')' | '|' => { return Err(Error::IllegalChar(first_char + i)); }
                _ => {}
            }
        }

        let chars = match chars {
            Some(c) if c.is_empty() => { return Err(Error::EmptyRange(start)); }
            None => { return Err(Error::UnclosedRange(self.chars.len() - 1)); }
            Some(c) => c,
        };

        self.i = first_char + chars.len() + 1;

        if negated {
            Ok(Token::NotAnyOf(parse_char_specifiers(chars)))
        } else {
            Ok(Token::AnyOf(parse_char_specifiers(chars)))
        }
    }

    fn parse_patterns(&mut self) -> Result<Vec<Vec<Token>>, Error> {
        let start = self.i + 2;
        let mut paren_stack = Vec::new();

        let mut escaped = false;
        let mut chars = None;
        for (i, c) in self.chars[start..].iter().enumerate() {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                ']' => {
                    match paren_stack.last() {
                        Some(c) if *c != '[' => return Err(Error::IllegalChar(start + i)),
                        None => return Err(Error::IllegalChar(start + i)),
                        _ => paren_stack.pop(),
                    };
                }
                ')' if paren_stack.is_empty() => {
                    chars = Some(&self.chars[start..start + i]);
                    break;
                }
                ')' => {
                    match paren_stack.last() {
                        Some(c) if *c != '(' => return Err(Error::IllegalChar(start + i)),
                        None => return Err(Error::IllegalChar(start + i)),
                        _ => paren_stack.pop(),
                    };
                }
                '(' | '[' => paren_stack.push(*c),
                _ => {}
            }
        }

        let chars = match chars {
            Some(c) if c.is_empty() => { return Err(Error::EmptyPattern(start)); }
            None => { return Err(Error::UnclosedPattern(self.chars.len() - 1)); }
            Some(c) => c,
        };

        let mut pattern_parts = Vec::new();
        let mut last_pattern = 0;
        for (i, c) in chars.iter().enumerate() {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                ']' => {
                    match paren_stack.last() {
                        Some(c) if *c != '[' => return Err(Error::IllegalChar(start + i)),
                        None => return Err(Error::IllegalChar(start + i)),
                        _ => paren_stack.pop(),
                    };
                }
                ')' => {
                    match paren_stack.last() {
                        Some(c) if *c != '(' => return Err(Error::IllegalChar(start + i)),
                        None => return Err(Error::IllegalChar(start + i)),
                        _ => paren_stack.pop(),
                    };
                }
                '(' | '[' => paren_stack.push(*c),
                '|' => if paren_stack.len() == 0 {
                    let part = &self.chars[start + last_pattern..start + i];
                    if part.is_empty() {
                        return Err(Error::IllegalOr(start + last_pattern));
                    }
                    last_pattern = i+1;
                    pattern_parts.push(part);
                }
                _ => {}
            }
        }

        let part = &self.chars[start + last_pattern..start + chars.len()];
        if part.is_empty() {
            return Err(Error::IllegalOr(start + last_pattern));
        }
        pattern_parts.push(part);

        let mut tokens = Vec::new();
        for part in pattern_parts.into_iter()   {
            let mut pattern = String::new();
            for c in part {
                pattern.push(*c)
            }
            tokens.push(parse(&pattern)?)
        }

        self.i = start + chars.len()+1;

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
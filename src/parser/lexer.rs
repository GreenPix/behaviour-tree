use std::str::Chars;

#[derive(Debug)]
pub enum Token {
    Ident(String),
    QuotedString(String),
    Integer(i64),
    Root,
    Subtree,
    Selector,
    Sequence,
    Priority,
    Inverter,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    LeftParenthesis,
    RightParenthesis,
    LeftArray,
    RightArray,
    Plus,
    Minus,
    Multiply,
    Divide,
}

struct Memory<T: Iterator> {
    inner: T,
    last_item: Option<<T as Iterator>::Item>,
    rewind: bool,
}

impl <I,T> Iterator for Memory<T>
where I: Copy,
      T: Iterator<Item=I> {
    type Item = <T as Iterator>::Item;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if !self.rewind {
            self.last_item = self.inner.next();
        } else {
            self.rewind = false;
        }
        self.last_item
    }
}

impl <I,T> Memory<T>
where I: Copy,
      T: Iterator<Item=I> {
    fn new(iter: T) -> Memory<T> {
        Memory {
            inner: iter,
            last_item: None,
            rewind: false,
        }
    }

    fn rewind(&mut self) {
        if !self.rewind {
            self.rewind = true;
        }
        else {
            panic!("The lexer was already rewinding, must be a bug");
        }
    }

    fn previous(&self) -> Option<<Self as Iterator>::Item> {
        self.last_item
    }
}

pub struct Tokenizer<'a> {
    inner: Memory<Chars<'a>>,
}

impl <'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token,String>;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        self.consume_whitespace();
        let next = match self.inner.next() {
            None => return None,
            Some(c) => c,
        };
        let token = match next {
            '{' => Token::LeftBracket,
            '}' => Token::RightBracket,
            ',' => Token::Comma,
            ':' => Token::Colon,
            '(' => Token::LeftParenthesis,
            ')' => Token::RightParenthesis,
            '[' => Token::LeftArray,
            ']' => Token::RightArray,
            '+' => Token::Plus,
            '*' => Token::Multiply,
            '/' => Token::Divide,
            c if c.is_alphabetic() => {
                self.inner.rewind();
                self.parse_word()
            }
            c if c == '"' => {
                match self.parse_quoted_string() {
                    Ok(token) => token,
                    Err(e) => return Some(Err(e)),
                }
            }
            c if c.is_numeric() => {
                self.inner.rewind();
                Token::Integer(self.parse_number())
            }
            '-' => {
                // Special case for - : it can be an operator in an expression or a negative number
                // They can be differenciated by the following character
                match self.inner.next() {
                    Some(c) if c.is_numeric() => {
                        // Negative number
                        self.inner.rewind();
                        Token::Integer(-self.parse_number())
                    }
                    _ => {
                        self.inner.rewind();
                        Token::Minus
                    }
                }
            }
            other => return Some(Err(format!("Unrecognized character {}", other))),
        };
        Some(Ok(token))
    }
}

impl <'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            inner: Memory::new(input.chars())
        }
    }

    fn consume_whitespace(&mut self) {
        for _ in self.inner.by_ref().take_while(|&c| c.is_whitespace()) {}
        self.inner.rewind();
    }

    fn parse_word(&mut self) -> Token {
        let word: String = self.inner.by_ref().take_while(is_valid_id).collect();
        self.inner.rewind();
        match word.as_ref() {
            "tree" => return Token::Root,
            "subtree" => return Token::Subtree,
            "selector" => return Token::Selector,
            "sequence" => return Token::Sequence,
            "inverter" => return Token::Inverter,
            "priority" => return Token::Priority,
            _ => {}
        }
        assert!(word.len() != 0);
        Token::Ident(word)
    }

    fn parse_number(&mut self) -> i64 {
        let number_str: String = self.inner.by_ref().take_while(|&c| c.is_numeric()).collect();
        self.inner.rewind();
        let number = i64::from_str_radix(&number_str, 10).unwrap();
        number
    }

    fn parse_quoted_string(&mut self) -> Result<Token,String> {
        let mut res = String::new();
        loop {
            res.extend(self.inner.by_ref().take_while(|&c| c != '"' && c != '\\'));
            match self.inner.previous() {
                Some('\\') => {
                    match self.inner.next() {
                        Some('\\') => res.push('\\'),
                        Some('n') => res.push('\n'),
                        Some('"') => res.push('"'),
                        Some('t') => res.push('\t'),
                        Some(other) => {
                            println!("Lexer: unnecessary escape for character {}", other);
                            res.push(other);
                        }
                        None => {
                            return Err(String::from("Lexer error: unfinished quoted string during escape sequence"));
                        }
                    }
                },
                Some('"') => break,
                Some(..) => {
                    println!("Oops, a weird thing happened");
                    break;
                }
                None => {
                    return Err(String::from("Lexer error: unfinished quoted string"));
                }
            }
        }
        Ok(Token::QuotedString(res))
    }
}

fn is_valid_id(&c: &char) -> bool {
    c.is_alphanumeric() || c == '_'
}


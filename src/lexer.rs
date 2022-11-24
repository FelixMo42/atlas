use crate::core::Value;

#[derive(PartialEq)]
pub enum Token {
    Op(char),
    Value(Value),
    Err,
}

pub fn get_token(src: &str) -> (Token, usize) {
    let mut step = 0;
    let mut len = 0;

    loop {
        // get the next character
        let chr = src[len..].chars().next().unwrap_or('\x00');

        step = match step {
            0 /* start */ => {
                match chr {
                    '-' => return (Token::Op('-'), 1),
                    '+' => return (Token::Op('+'), 1),
                    '*' => return (Token::Op('*'), 1),
                    '/' => return (Token::Op('/'), 1),
                    '(' => return (Token::Op('('), 1),
                    ')' => return (Token::Op(')'), 1),
                    '0'..='9' => 1,
                    '.' => 3,
                    _ => return (Token::Err, 1),
                }
            }
            1 /* number */ => {
                match chr {
                    '0'..='9' => 1,
                    '.' => 2,
                    _ => return (Token::Value(Value::I32(src[..len]
                        .parse()
                        .expect("unexpecter error parsing int token")
                    )), len)
                }
            }
            2 /* float */ => {
                match chr {
                    '0'..='9' => 2,
                    _ => return (Token::Value(Value::F64(src[..len]
                        .parse()
                        .expect("unexpected error parsing float token")
                    )), len),
                }
            }
            3 /* dot or float */ => {
                match chr {
                    '0'..='9' => 3,
                    _ => return (Token::Err, 1),
                }
            }
            _ => unreachable!()
        };

        // incrament the length
        len += 1;
    }
}

pub struct Lexer<'a> {
    src: &'a str,
    index: usize,
}

fn calc_whitespace(src: &str, index: usize) -> usize {
    if index >= src.len() {
        0
    } else {
        src[index..]
            .chars()
            .take_while(|c| c.is_whitespace())
            .count()
    }
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        return Lexer {
            src: src.trim(),
            index: 0,
        };
    }

    pub fn save(&self) -> usize {
        return self.index;
    }

    pub fn next(&mut self) -> Token {
        let (tok, len) = get_token(&self.src[self.index..]);

        self.index += len + calc_whitespace(&self.src, self.index + len);

        return tok;
    }

    pub fn load(&mut self, index: usize) {
        self.index = index
    }

    pub fn log(&self) {
        println!("{}~{}", &self.src[..self.index], &self.src[self.index..]);
    }
}

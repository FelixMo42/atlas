#[derive(PartialEq, Clone, Copy)]
pub enum Token<'a> {
    Comment,
    Ident(&'a str),
    Set,
    Err,

    // literals
    I32(i32),
    F64(f64),

    // punctuation
    Open(char),
    Close(char),
    Comma,

    // mathmatical operator
    Add,
    Sub,
    Mul,
    Div,

    // comparison
    Eq,
    Gt,
    Lt,
    Ge,
    Le,
    Ne,
}

pub fn parse_token(src: &str) -> (Token, usize) {
    let mut step = 0;
    let mut len = 0;

    loop {
        // get the next character
        let chr = src[len..].chars().next().unwrap_or('\x00');

        step = match step {
            0 /* start */ => {
                match chr {
                    '-' => return (Token::Sub, 1),
                    '+' => return (Token::Add, 1),
                    '*' => return (Token::Mul, 1),
                    '/' => 9,
                    '(' => return (Token::Open('('), 1),
                    ')' => return (Token::Close(')'), 1),
                    '{' => return (Token::Open('{'), 1),
                    '}' => return (Token::Close('}'), 1),
                    '[' => return (Token::Open('['), 1),
                    ']' => return (Token::Close(']'), 1),
                    ',' => return (Token::Comma, 1),
                    '=' => 5,
                    '<' => 6,
                    '>' => 7,
                    '!' => 8,
                    '0'..='9' => 1,
                    'a'..='z' | 'A'..='Z' | '_' => 4,
                    '.' => 3,
                    _ => return (Token::Err, 1),
                }
            }
            1 /* number */ => match chr {
                '0'..='9' => 1,
                '.' => 2,
                _ => return (Token::I32(src[..len]
                    .parse()
                    .expect("unexpecter error parsing int token")
                ), len)
            },
            2 /* float */ => match chr {
                '0'..='9' => 2,
                _ => return (Token::F64(src[..len]
                    .parse()
                    .expect("unexpected error parsing float token")
                ), len),
            },
            3 /* dot or float */ => match chr {
                '0'..='9' => 3,
                _ => return (Token::Err, 1),
            },
            4 /* ident */ => match chr {
                'a'..='z' | 'A'..='Z' | '_' => 4,
                _ => return (Token::Ident(&src[..len]), len),
            },
            5 /* equal */ => match chr {
                '=' => return (Token::Eq, 2),
                _ => return (Token::Set, 1),
            },
            6 /* greater than */ => match chr {
                '=' => return (Token::Le, 2),
                _ => return (Token::Lt, 1),
            },
            7 /* less than */ => match chr {
                '=' => return (Token::Ge, 2),
                _ => return (Token::Gt, 1),
            },
            8 /* not */ => match chr {
                '=' => return (Token::Ne, 2),
                _ => return (Token::Err, 1),
            },
            9 /* maybe comment */ => match chr {
                '/' => 10,
                _ => return (Token::Div, 1)
            }
            10 /* comment */ => match chr {
                '\n' | '\x00' => return (Token::Comment, len),
                _ => 10
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
    token: Token<'a>,
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
            token: Token::Err,
        };
    }

    pub fn save(&self) -> usize {
        return self.index;
    }

    pub fn next(&mut self) -> Token {
        self._next();
        while self.token == Token::Comment {
            self._next();
        }
        return self.token;
    }

    fn _next(&mut self) {
        let (tok, len) = parse_token(&self.src[self.index..]);
        self.index += len + calc_whitespace(&self.src, self.index + len);
        self.token = tok;
    }

    pub fn load(&mut self, index: usize) {
        self.index = index
    }
}

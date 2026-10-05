use crate::model::cell::CellCoord;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Power,
    Concat,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Pos,
    Percent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Text(String),
    Bool(bool),
    CellRef(CellCoord),
    RangeRef { start: CellCoord, end: CellCoord },
    Unary { op: UnaryOp, expr: Box<Expr> },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    FunctionCall { name: String, args: Vec<Expr> },
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    StringLit(String),
    Ident(String),
    CellOrRange(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Ampersand,
    Percent,
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    LParen,
    RParen,
    Comma,
    Colon,
}

struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    fn tokenize(mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
                continue;
            }
            match c {
                '+' => {
                    self.chars.next();
                    tokens.push(Token::Plus);
                }
                '-' => {
                    self.chars.next();
                    tokens.push(Token::Minus);
                }
                '*' => {
                    self.chars.next();
                    tokens.push(Token::Star);
                }
                '/' => {
                    self.chars.next();
                    tokens.push(Token::Slash);
                }
                '^' => {
                    self.chars.next();
                    tokens.push(Token::Caret);
                }
                '&' => {
                    self.chars.next();
                    tokens.push(Token::Ampersand);
                }
                '%' => {
                    self.chars.next();
                    tokens.push(Token::Percent);
                }
                '(' => {
                    self.chars.next();
                    tokens.push(Token::LParen);
                }
                ')' => {
                    self.chars.next();
                    tokens.push(Token::RParen);
                }
                ',' => {
                    self.chars.next();
                    tokens.push(Token::Comma);
                }
                ':' => {
                    self.chars.next();
                    tokens.push(Token::Colon);
                }
                '=' => {
                    self.chars.next();
                    tokens.push(Token::Equal);
                }
                '<' => {
                    self.chars.next();
                    if let Some(&'>') = self.chars.peek() {
                        self.chars.next();
                        tokens.push(Token::NotEqual);
                    } else if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        tokens.push(Token::LessEqual);
                    } else {
                        tokens.push(Token::LessThan);
                    }
                }
                '>' => {
                    self.chars.next();
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        tokens.push(Token::GreaterEqual);
                    } else {
                        tokens.push(Token::GreaterThan);
                    }
                }
                '"' => {
                    self.chars.next();
                    let mut s = String::new();
                    let mut closed = false;
                    while let Some(ch) = self.chars.next() {
                        if ch == '"' {
                            if let Some(&'"') = self.chars.peek() {
                                self.chars.next();
                                s.push('"');
                            } else {
                                closed = true;
                                break;
                            }
                        } else {
                            s.push(ch);
                        }
                    }
                    if !closed {
                        return Err("Unterminated string literal in formula".to_string());
                    }
                    tokens.push(Token::StringLit(s));
                }
                _ if c.is_ascii_digit() || c == '.' => {
                    let mut num_str = String::new();
                    let mut has_dot = false;
                    while let Some(&ch) = self.chars.peek() {
                        if ch.is_ascii_digit() {
                            num_str.push(ch);
                            self.chars.next();
                        } else if ch == '.' && !has_dot {
                            has_dot = true;
                            num_str.push(ch);
                            self.chars.next();
                        } else {
                            break;
                        }
                    }
                    let num = num_str
                        .parse::<f64>()
                        .map_err(|_| format!("Invalid number: {}", num_str))?;
                    tokens.push(Token::Number(num));
                }
                _ if c.is_ascii_alphabetic() || c == '_' || c == '$' => {
                    let mut ident = String::new();
                    while let Some(&ch) = self.chars.peek() {
                        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' {
                            ident.push(ch);
                            self.chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Ident(ident));
                }
                _ => {
                    return Err(format!("Unexpected character in formula: '{}'", c));
                }
            }
        }
        Ok(tokens)
    }
}

pub struct FormulaParser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> FormulaParser<'a> {
    pub fn parse(input: &str) -> Result<Expr, String> {
        let clean = input.trim();
        let formula = if clean.starts_with('=') {
            &clean[1..]
        } else {
            clean
        };
        let tokens = Lexer::new(formula).tokenize()?;
        let mut parser = FormulaParser {
            tokens: &tokens,
            pos: 0,
        };
        let expr = parser.parse_expr()?;
        if parser.pos < parser.tokens.len() {
            return Err("Unexpected token after formula expression".to_string());
        }
        Ok(expr)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos);
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_concat()?;
        while let Some(tok) = self.peek() {
            let op = match tok {
                Token::Equal => BinaryOp::Eq,
                Token::NotEqual => BinaryOp::NotEq,
                Token::LessThan => BinaryOp::Lt,
                Token::LessEqual => BinaryOp::LtEq,
                Token::GreaterThan => BinaryOp::Gt,
                Token::GreaterEqual => BinaryOp::GtEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_concat()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_concat(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_additive()?;
        while let Some(Token::Ampersand) = self.peek() {
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::Binary {
                op: BinaryOp::Concat,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplicative()?;
        while let Some(tok) = self.peek() {
            let op = match tok {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_power()?;
        while let Some(tok) = self.peek() {
            let op = match tok {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_power()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_power(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        if let Some(Token::Caret) = self.peek() {
            self.advance();
            let right = self.parse_power()?; // Right-associative
            left = Expr::Binary {
                op: BinaryOp::Power,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if let Some(tok) = self.peek() {
            match tok {
                Token::Minus => {
                    self.advance();
                    let inner = self.parse_unary()?;
                    return Ok(Expr::Unary {
                        op: UnaryOp::Neg,
                        expr: Box::new(inner),
                    });
                }
                Token::Plus => {
                    self.advance();
                    return self.parse_unary();
                }
                _ => {}
            }
        }

        let mut primary = self.parse_primary()?;

        // Postfix operators like %
        while let Some(Token::Percent) = self.peek() {
            self.advance();
            primary = Expr::Unary {
                op: UnaryOp::Percent,
                expr: Box::new(primary),
            };
        }

        Ok(primary)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let tok = self
            .advance()
            .ok_or_else(|| "Unexpected end of formula".to_string())?
            .clone();

        match tok {
            Token::Number(n) => Ok(Expr::Number(n)),
            Token::StringLit(s) => Ok(Expr::Text(s)),
            Token::LParen => {
                let inner = self.parse_expr()?;
                if let Some(Token::RParen) = self.advance() {
                    Ok(inner)
                } else {
                    Err("Missing closing parenthesis ')'".to_string())
                }
            }
            Token::Ident(ident) => {
                let upper = ident.to_ascii_uppercase();
                if upper == "TRUE" {
                    return Ok(Expr::Bool(true));
                }
                if upper == "FALSE" {
                    return Ok(Expr::Bool(false));
                }

                // Check if followed by '(' -> Function call
                if let Some(Token::LParen) = self.peek() {
                    self.advance(); // consume '('
                    let mut args = Vec::new();
                    if let Some(Token::RParen) = self.peek() {
                        self.advance(); // consume ')'
                        return Ok(Expr::FunctionCall {
                            name: upper,
                            args,
                        });
                    }

                    loop {
                        let arg = self.parse_expr()?;
                        args.push(arg);

                        match self.peek() {
                            Some(Token::Comma) => {
                                self.advance();
                            }
                            Some(Token::RParen) => {
                                self.advance();
                                break;
                            }
                            _ => return Err("Expected ',' or ')' in argument list".to_string()),
                        }
                    }

                    return Ok(Expr::FunctionCall {
                        name: upper,
                        args,
                    });
                }

                // Check if this is a cell coordinate or range: e.g. A1 or A1:B10
                if let Ok(coord) = CellCoord::from_a1(&ident) {
                    if let Some(Token::Colon) = self.peek() {
                        self.advance(); // consume ':'
                        if let Some(Token::Ident(end_ident)) = self.advance() {
                            let end_coord = CellCoord::from_a1(end_ident)?;
                            return Ok(Expr::RangeRef {
                                start: coord,
                                end: end_coord,
                            });
                        } else {
                            return Err("Expected cell reference after ':' in range".to_string());
                        }
                    }
                    return Ok(Expr::CellRef(coord));
                }

                Err(format!("Unrecognized identifier or reference: {}", ident))
            }
            _ => Err("Unexpected token in formula".to_string()),
        }
    }
}

/// Recursively extract all cell references and ranges referenced by an expression
pub fn extract_dependencies(expr: &Expr) -> Vec<CellCoord> {
    let mut deps = Vec::new();
    collect_dependencies(expr, &mut deps);
    deps.sort();
    deps.dedup();
    deps
}

fn collect_dependencies(expr: &Expr, out: &mut Vec<CellCoord>) {
    match expr {
        Expr::CellRef(coord) => {
            out.push(*coord);
        }
        Expr::RangeRef { start, end } => {
            let min_r = start.row.min(end.row);
            let max_r = start.row.max(end.row);
            let min_c = start.col.min(end.col);
            let max_c = start.col.max(end.col);
            for r in min_r..=max_r {
                for c in min_c..=max_c {
                    out.push(CellCoord::new(r, c));
                }
            }
        }
        Expr::Unary { expr, .. } => {
            collect_dependencies(expr, out);
        }
        Expr::Binary { left, right, .. } => {
            collect_dependencies(left, out);
            collect_dependencies(right, out);
        }
        Expr::FunctionCall { args, .. } => {
            for arg in args {
                collect_dependencies(arg, out);
            }
        }
        Expr::Number(_) | Expr::Text(_) | Expr::Bool(_) => {}
    }
}

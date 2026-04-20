//! JSON token lexer expressed as a state machine.
//!
//! Maps a stream of `char`s to a stream of `Vec<Token>` — usually empty,
//! one on most delimiters, occasionally two when flushing a number/keyword
//! whose terminator is itself a token (e.g. `42,` emits `Number("42")`
//! and `Comma` in the same step).
//!
//! Scope is deliberately narrow to keep the example tight: no escape
//! sequences inside strings, no strict number validation (accepts anything
//! vaguely numeric until a delimiter), no Unicode. Add those and you have
//! a real JSON lexer.

use state_machines_rs::{Runner, StateMachine};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Colon,
    Comma,
    Number(String),
    Str(String),
    True,
    False,
    Null,
    Error(char),
}

#[derive(Clone, Debug)]
pub enum LexState {
    Start,
    InNumber(String),
    InString(String),
    /// Matching a keyword; `buf` holds what we've seen, `target` is the full
    /// literal we expect (`"true"`, `"false"`, or `"null"`).
    InKeyword { buf: String, target: &'static str },
}

pub struct JsonLexer;

fn delimiter_token(c: char) -> Option<Token> {
    match c {
        '{' => Some(Token::LBrace),
        '}' => Some(Token::RBrace),
        '[' => Some(Token::LBracket),
        ']' => Some(Token::RBracket),
        ':' => Some(Token::Colon),
        ',' => Some(Token::Comma),
        _ => None,
    }
}

fn is_number_char(c: char) -> bool {
    c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E')
}

/// Process `c` assuming we're in `Start`. Returns (next_state, tokens).
fn start_step(c: char) -> (LexState, Vec<Token>) {
    if c.is_whitespace() {
        return (LexState::Start, vec![]);
    }
    if let Some(t) = delimiter_token(c) {
        return (LexState::Start, vec![t]);
    }
    if c == '"' {
        return (LexState::InString(String::new()), vec![]);
    }
    if c.is_ascii_digit() || c == '-' {
        return (LexState::InNumber(c.to_string()), vec![]);
    }
    match c {
        't' => (LexState::InKeyword { buf: "t".into(), target: "true" }, vec![]),
        'f' => (LexState::InKeyword { buf: "f".into(), target: "false" }, vec![]),
        'n' => (LexState::InKeyword { buf: "n".into(), target: "null" }, vec![]),
        _ => (LexState::Start, vec![Token::Error(c)]),
    }
}

impl StateMachine for JsonLexer {
    type Input = char;
    type Output = Vec<Token>;
    type State = LexState;

    fn start_state(&self) -> LexState {
        LexState::Start
    }

    fn next_values(&self, state: &LexState, c: &char) -> (LexState, Vec<Token>) {
        let c = *c;
        match state {
            LexState::Start => start_step(c),

            LexState::InNumber(buf) => {
                if is_number_char(c) {
                    let mut b = buf.clone();
                    b.push(c);
                    (LexState::InNumber(b), vec![])
                } else {
                    // Delimiter — flush the number, then re-process c from Start.
                    let (next, mut toks) = start_step(c);
                    toks.insert(0, Token::Number(buf.clone()));
                    (next, toks)
                }
            }

            LexState::InString(buf) => {
                if c == '"' {
                    (LexState::Start, vec![Token::Str(buf.clone())])
                } else {
                    let mut b = buf.clone();
                    b.push(c);
                    (LexState::InString(b), vec![])
                }
            }

            LexState::InKeyword { buf, target } => {
                let mut b = buf.clone();
                b.push(c);
                if b == *target {
                    let token = match *target {
                        "true" => Token::True,
                        "false" => Token::False,
                        "null" => Token::Null,
                        _ => unreachable!(),
                    };
                    (LexState::Start, vec![token])
                } else if target.starts_with(&b) {
                    (LexState::InKeyword { buf: b, target }, vec![])
                } else {
                    (LexState::Start, vec![Token::Error(c)])
                }
            }
        }
    }
}

fn main() {
    // Trailing space flushes any pending number/keyword.
    let source = r#"{"name": "ada", "age": 42, "active": true, "score": -3.14e2, "tags": [null, 1] } "#;

    let mut tokens: Vec<Token> = Runner::new(JsonLexer)
        .transduce(source.chars())
        .into_iter()
        .flatten()
        .collect();

    for t in &tokens {
        println!("{:?}", t);
    }

    // Strip out any trailing Error tokens (there shouldn't be any on valid input).
    tokens.retain(|t| !matches!(t, Token::Error(_)));

    use Token::*;
    let expected = vec![
        LBrace,
        Str("name".into()), Colon, Str("ada".into()), Comma,
        Str("age".into()), Colon, Number("42".into()), Comma,
        Str("active".into()), Colon, True, Comma,
        Str("score".into()), Colon, Number("-3.14e2".into()), Comma,
        Str("tags".into()), Colon, LBracket, Null, Comma, Number("1".into()), RBracket,
        RBrace,
    ];
    assert_eq!(tokens, expected);
}

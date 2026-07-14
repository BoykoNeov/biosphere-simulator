//! Parse a rate-expression string into a [`simcore::expr::Expr`] AST — the Rust port
//! of Python `authoring.expr_parser` (Phase 9, Step 4a).
//!
//! Parsing text → AST is a one-time **boundary** act (decision A): it runs when a
//! scenario file is interpreted, never per step, so it lives here in the `authoring`
//! crate and not in the pure `simcore` core (which only *evaluates* the AST).
//!
//! This is **the sole Tier-0 parse-parity surface** the plan flags: the grammar's
//! precedence and associativity must match the Python parser exactly, so the same rate
//! string lowers to the same AST in both ports. The grammar is the bounded arithmetic
//! core Step 2 ships (see [`simcore::expr`] for the closed-set rationale):
//!
//! ```text
//! expr    := term  (("+" | "-") term)*         # left-associative
//! term    := factor ("*" factor)*              # left-associative
//! factor  := "-" factor | primary              # unary minus binds tighter than "*"
//! primary := number
//!          | "stock"   "(" string ")"
//!          | "param"   "(" string ")"
//!          | "forcing" "(" string ")"
//!          | "n"
//!          | "(" expr ")"
//! ```
//!
//! `number` is standard decimal/float syntax (`1`, `1.5`, `1.0e-8` — parsed by *this*
//! parser, so the YAML-1.1 dotless-`1e-3`-is-a-string hazard does not apply here).
//! `f64::from_str`, like Python `float()`, is correctly-rounded, so a decimal literal
//! lowers to the identical bits in both ports (the Option-C round-trip assumption, now
//! resting on `Const` parity). A `stock`/`param`/`forcing` argument is a **quoted
//! string** id/name, so any dotted id (`power.battery`) is expressible.
//!
//! **The deferred grammar is deferred here too (do NOT complete it).** An unsupported
//! operator such as `/`, an unknown identifier, an unbalanced paren, or trailing junk
//! is a [`ParseError`] — exactly as the Python parser raises `AuthoringError`. The
//! error *messages* are deliberately **not** pinned cross-port (Tier-0 parity is
//! structural: accept→same AST, reject→both error).

use std::fmt;

use simcore::expr::{BinaryOp, Expr};

/// A rate-expression parse failure. Carries a human message + the offending code-point
/// position; the message text is a diagnostic only (never a cross-port parity target).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub pos: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (at position {})", self.message, self.pos)
    }
}

impl std::error::Error for ParseError {}

/// A lexical token. `kind` mirrors the Python tokenizer's tag set.
#[derive(Debug, Clone, PartialEq)]
enum TokKind {
    Num,
    Ident,
    Str,
    Plus,
    Minus,
    Star,
    LParen,
    RParen,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokKind,
    value: String,
    pos: usize,
}

/// The three reference forms — the closed keyword set (`n` is handled separately).
const REF_KEYWORDS: [&str; 3] = ["stock", "param", "forcing"];

/// Split `text` into tokens over its char sequence (code-point positions mirror
/// Python's `text[i]` indexing). A stray character is a [`ParseError`].
fn tokenize(text: &str) -> Result<Vec<Token>, ParseError> {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut tokens: Vec<Token> = Vec::new();
    let mut i = 0;
    while i < n {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        match c {
            '+' => {
                tokens.push(Token { kind: TokKind::Plus, value: "+".into(), pos: i });
                i += 1;
                continue;
            }
            '-' => {
                tokens.push(Token { kind: TokKind::Minus, value: "-".into(), pos: i });
                i += 1;
                continue;
            }
            '*' => {
                tokens.push(Token { kind: TokKind::Star, value: "*".into(), pos: i });
                i += 1;
                continue;
            }
            '(' => {
                tokens.push(Token { kind: TokKind::LParen, value: "(".into(), pos: i });
                i += 1;
                continue;
            }
            ')' => {
                tokens.push(Token { kind: TokKind::RParen, value: ")".into(), pos: i });
                i += 1;
                continue;
            }
            '\'' | '"' => {
                // A quoted string literal (the ref argument). No escapes — an id/var
                // name is simple ASCII, so it ends at the next matching quote.
                let quote = c;
                let mut end = None;
                for (j, cj) in chars.iter().enumerate().skip(i + 1) {
                    if *cj == quote {
                        end = Some(j);
                        break;
                    }
                }
                match end {
                    None => {
                        return Err(ParseError {
                            message: format!("unterminated string literal in {text:?}"),
                            pos: i,
                        })
                    }
                    Some(end) => {
                        let s: String = chars[i + 1..end].iter().collect();
                        tokens.push(Token { kind: TokKind::Str, value: s, pos: i });
                        i = end + 1;
                        continue;
                    }
                }
            }
            _ => {}
        }
        if c.is_ascii_digit() || c == '.' {
            let start = i;
            // A permissive numeric scan (digits, one dot, an e/E exponent with optional
            // sign); f64::from_str is the final arbiter of validity. Only consume a sign
            // when it is part of an exponent (previous char e/E).
            while i < n && (chars[i].is_ascii_digit() || matches!(chars[i], '.' | 'e' | 'E' | '+' | '-')) {
                if matches!(chars[i], '+' | '-') && (i == start || !matches!(chars[i - 1], 'e' | 'E')) {
                    break;
                }
                i += 1;
            }
            let literal: String = chars[start..i].iter().collect();
            if literal.parse::<f64>().is_err() {
                return Err(ParseError {
                    message: format!("invalid number {literal:?} in {text:?}"),
                    pos: start,
                });
            }
            tokens.push(Token { kind: TokKind::Num, value: literal, pos: start });
            continue;
        }
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < n && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let ident: String = chars[start..i].iter().collect();
            tokens.push(Token { kind: TokKind::Ident, value: ident, pos: start });
            continue;
        }
        return Err(ParseError {
            message: format!(
                "unexpected character {c:?} in {text:?} (the rate grammar is arithmetic \
                 + stock/param/forcing/n only)"
            ),
            pos: i,
        });
    }
    Ok(tokens)
}

/// A single-use recursive-descent parser over the token stream.
struct Parser {
    tokens: Vec<Token>,
    text: String,
    i: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.i)
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens[self.i].clone();
        self.i += 1;
        tok
    }

    fn error(&self, msg: impl Into<String>, pos: usize) -> ParseError {
        ParseError {
            message: format!("{} in rate expression {:?}", msg.into(), self.text),
            pos,
        }
    }

    fn parse(&mut self) -> Result<Expr, ParseError> {
        let expr = self.expr()?;
        if let Some(extra) = self.peek() {
            return Err(self.error(
                format!("unexpected trailing token {:?}", extra.value),
                extra.pos,
            ));
        }
        Ok(expr)
    }

    fn expr(&mut self) -> Result<Expr, ParseError> {
        let mut node = self.term()?;
        loop {
            let op = match self.peek() {
                Some(t) if t.kind == TokKind::Plus => BinaryOp::Add,
                Some(t) if t.kind == TokKind::Minus => BinaryOp::Sub,
                _ => return Ok(node),
            };
            self.advance();
            let right = self.term()?;
            node = Expr::BinOp { op, left: Box::new(node), right: Box::new(right) };
        }
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut node = self.factor()?;
        loop {
            match self.peek() {
                Some(t) if t.kind == TokKind::Star => {
                    self.advance();
                    let right = self.factor()?;
                    node = Expr::BinOp {
                        op: BinaryOp::Mul,
                        left: Box::new(node),
                        right: Box::new(right),
                    };
                }
                _ => return Ok(node),
            }
        }
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        if let Some(t) = self.peek() {
            if t.kind == TokKind::Minus {
                self.advance();
                return Ok(Expr::Neg(Box::new(self.factor()?)));
            }
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        let tok = match self.peek() {
            None => return Err(self.error("unexpected end of expression (expected a value)", self.text.chars().count())),
            Some(t) => t.clone(),
        };
        match tok.kind {
            TokKind::Num => {
                self.advance();
                // Validated in the tokenizer; from_str is correctly-rounded (== Python).
                Ok(Expr::Const(tok.value.parse::<f64>().unwrap()))
            }
            TokKind::LParen => {
                self.advance();
                let node = self.expr()?;
                match self.peek() {
                    Some(t) if t.kind == TokKind::RParen => {
                        self.advance();
                        Ok(node)
                    }
                    _ => Err(self.error(format!("missing ')' (opened at position {})", tok.pos), tok.pos)),
                }
            }
            TokKind::Ident => self.ident(&tok),
            _ => Err(self.error(format!("unexpected token {:?}", tok.value), tok.pos)),
        }
    }

    fn ident(&mut self, tok: &Token) -> Result<Expr, ParseError> {
        self.advance();
        if tok.value == "n" {
            return Ok(Expr::StepN);
        }
        if REF_KEYWORDS.contains(&tok.value.as_str()) {
            let arg = self.call_string_arg(tok)?;
            return Ok(match tok.value.as_str() {
                "stock" => Expr::StockRef(arg),
                "param" => Expr::ParamRef(arg),
                _ => Expr::ForcingRef(arg),
            });
        }
        Err(self.error(
            format!(
                "unknown identifier {:?} (expected a number, 'n', or \
                 stock/param/forcing(\"…\"))",
                tok.value
            ),
            tok.pos,
        ))
    }

    /// Consume `( "string" )` after a `stock`/`param`/`forcing` keyword.
    fn call_string_arg(&mut self, keyword: &Token) -> Result<String, ParseError> {
        match self.peek() {
            Some(t) if t.kind == TokKind::LParen => self.advance(),
            _ => return Err(self.error(format!("expected '(' after {:?}", keyword.value), keyword.pos)),
        };
        let arg = match self.peek() {
            Some(t) if t.kind == TokKind::Str => self.advance(),
            _ => {
                return Err(self.error(
                    format!("{}(…) takes a quoted string argument", keyword.value),
                    keyword.pos,
                ))
            }
        };
        match self.peek() {
            Some(t) if t.kind == TokKind::RParen => self.advance(),
            _ => {
                return Err(self.error(
                    format!("missing ')' after {}(…)", keyword.value),
                    keyword.pos,
                ))
            }
        };
        if arg.value.is_empty() {
            return Err(self.error(format!("{}(…) argument is empty", keyword.value), keyword.pos));
        }
        Ok(arg.value)
    }
}

/// Parse a rate-expression string into a [`simcore::expr::Expr`] AST — the single
/// public entry point (Python `parse_rate_expr`). Raises [`ParseError`] for any
/// malformed input (empty expression, stray character, unknown identifier, unbalanced
/// parens, an unsupported operator, trailing junk).
pub fn parse_rate_expr(text: &str) -> Result<Expr, ParseError> {
    let tokens = tokenize(text)?;
    if tokens.is_empty() {
        return Err(ParseError {
            message: format!("empty rate expression {text:?}"),
            pos: 0,
        });
    }
    Parser {
        tokens,
        text: text.to_string(),
        i: 0,
    }
    .parse()
}

/// Render an AST back to a rate-expression string that [`parse_rate_expr`] round-trips
/// (`parse_rate_expr(render_rate_expr(node)) == node`) — the Rust mirror of Python
/// `authoring.expr_parser.render_rate_expr`.
///
/// Fully parenthesized, so the round-trip holds regardless of precedence/associativity.
/// The **inverse** of the parser, used by composition's id-namespacing (Step 6c): after
/// a bundle's `stock`/`forcing` refs are prefixed on the AST, the rate is re-emitted to a
/// string the interpreter re-parses. Its exact spelling is a **per-port internal detail**
/// (the structural graph dump omits rate strings; the trajectory depends only on the
/// AST), so the contract is per-port round-trip stability, **not** cross-port
/// byte-identity — a `Const` renders via `f64::Display`, which round-trips on this port
/// (Python uses `repr`, which round-trips on that one).
pub fn render_rate_expr(node: &Expr) -> String {
    match node {
        Expr::Const(v) => format!("{v}"),
        Expr::StockRef(id) => format!("stock(\"{id}\")"),
        Expr::ParamRef(name) => format!("param(\"{name}\")"),
        Expr::ForcingRef(name) => format!("forcing(\"{name}\")"),
        Expr::StepN => "n".to_string(),
        Expr::Neg(operand) => format!("(-{})", render_rate_expr(operand)),
        Expr::BinOp { op, left, right } => format!(
            "({} {} {})",
            render_rate_expr(left),
            op.symbol(),
            render_rate_expr(right)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sexpr::render_sexpr;

    fn sexpr(text: &str) -> String {
        render_sexpr(&parse_rate_expr(text).unwrap())
    }

    #[test]
    fn parses_the_self_discharge_rate() {
        assert_eq!(
            sexpr(r#"param("self_discharge_rate") * stock("power.battery")"#),
            r#"(* (param "self_discharge_rate") (stock "power.battery"))"#
        );
    }

    #[test]
    fn addition_is_left_associative() {
        // a - b - c parses as ((a - b) - c).
        assert_eq!(sexpr("n - n - n"), "(- (- n n) n)");
    }

    #[test]
    fn multiplication_binds_tighter_than_addition() {
        assert_eq!(sexpr("n + n * n"), "(+ n (* n n))");
    }

    #[test]
    fn unary_minus_binds_tighter_than_star() {
        // -n * n is (-n) * n, not -(n * n).
        assert_eq!(sexpr("-n * n"), "(* (neg n) n)");
    }

    #[test]
    fn parens_override_precedence() {
        assert_eq!(sexpr("(n + n) * n"), "(* (+ n n) n)");
    }

    #[test]
    fn reads_forcing_and_step_n() {
        assert_eq!(sexpr(r#"forcing("par") + n"#), r#"(+ (forcing "par") n)"#);
    }

    // --- the deferred grammar is rejected (do NOT complete it) -------------
    #[test]
    fn division_is_rejected() {
        assert!(parse_rate_expr("n / n").is_err());
    }

    #[test]
    fn unknown_identifier_is_rejected() {
        assert!(parse_rate_expr("exp(n)").is_err());
    }

    #[test]
    fn empty_expression_is_rejected() {
        assert!(parse_rate_expr("   ").is_err());
    }

    #[test]
    fn unbalanced_parens_are_rejected() {
        assert!(parse_rate_expr("(n + n").is_err());
    }

    #[test]
    fn trailing_junk_is_rejected() {
        assert!(parse_rate_expr("n n").is_err());
    }

    #[test]
    fn stray_character_is_rejected() {
        assert!(parse_rate_expr("n @ n").is_err());
    }

    #[test]
    fn unterminated_string_is_rejected() {
        assert!(parse_rate_expr(r#"stock("power.battery)"#).is_err());
    }

    #[test]
    fn empty_ref_argument_is_rejected() {
        assert!(parse_rate_expr(r#"stock("")"#).is_err());
    }

    // --- render_rate_expr is the parser's inverse (Step 6c namespacing) -----
    #[test]
    fn render_rate_expr_round_trips() {
        // Per-port round-trip stability: parse(render(parse(text))) == parse(text).
        // Fully parenthesized, so precedence/associativity survive; `Const` round-trips
        // through f64::Display on this port.
        for text in [
            "1.5",
            "1000.0",
            "1.0e-8",
            r#"stock("power.battery")"#,
            r#"param("self_discharge_rate") * stock("power.battery")"#,
            "1.0 + 2.0 * 3.0",
            "10.0 - 3.0 - 2.0",
            "(1.0 + 2.0) * 3.0",
            r#"-param("k") * n"#,
            r#"forcing("load_power") + n"#,
        ] {
            let ast = parse_rate_expr(text).unwrap();
            let rendered = render_rate_expr(&ast);
            assert_eq!(
                parse_rate_expr(&rendered).unwrap(),
                ast,
                "round-trip failed for {text:?} (rendered {rendered:?})"
            );
        }
    }
}

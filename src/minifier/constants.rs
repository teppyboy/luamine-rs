use full_moon::{ast::Expression, tokenizer::TokenReference};
use std::sync::LazyLock;

pub static EQ_TOKEN: LazyLock<Option<TokenReference>> =
    LazyLock::new(|| Some(TokenReference::symbol("=").unwrap()));
pub static NIL_SYMBOL: LazyLock<Expression> =
    LazyLock::new(|| Expression::Symbol(TokenReference::symbol("nil").unwrap()));
pub static COMMA: LazyLock<TokenReference> = LazyLock::new(|| TokenReference::symbol(",").unwrap());
pub static SEMICOLON: LazyLock<Option<TokenReference>> =
    LazyLock::new(|| Some(TokenReference::symbol(";").unwrap()));

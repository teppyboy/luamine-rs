use full_moon::{
    self, ShortString, ast::{Call, Expression, Index, Prefix, Suffix, span::ContainedSpan}, node::Node, tokenizer::{Token, TokenReference, TokenType}
};

use crate::minifier::Minifier;

pub fn trim_token(token: &mut Vec<&Token>) {
    let mut i = 0;
    println!("Initial token length: {}", token.len());
    while i < token.len() {
        println!("Current i: {}", i);
        match token[i].token_type() {
            TokenType::Whitespace { characters: _ } => {
                println!("Removing whitespace token: {:?}", token[i]);
                token.remove(i);
            }
            _ => {
                i += 1;
            }
        }
    }
    println!("Final token vector: {:#?}", token);
}

pub fn trim_leading(token_ref: &TokenReference) -> TokenReference {
    match token_ref.token().token_type() {
        TokenType::Whitespace { characters: _ } => {
            return TokenReference::symbol("").unwrap();
        }
        _ => {
            let (mut leading_trivia, trailing_trivia) = token_ref.surrounding_trivia();
            trim_token(&mut leading_trivia);
            TokenReference::new(
                leading_trivia.into_iter().cloned().collect(),
                token_ref.token().clone(),
                trailing_trivia.into_iter().cloned().collect(),
            )
        }
    }
}

pub fn trim(token_ref: &TokenReference) -> TokenReference {
    match token_ref.token().token_type() {
        TokenType::Whitespace { characters: _ } => {
            return TokenReference::symbol("").unwrap();
        }
        _ => {
            let (mut leading_trivia, mut trailing_trivia) = token_ref.surrounding_trivia();
            trim_token(&mut leading_trivia);
            trim_token(&mut trailing_trivia);
            TokenReference::new(
                leading_trivia.into_iter().cloned().collect(),
                token_ref.token().clone(),
                trailing_trivia.into_iter().cloned().collect(),
            )
        }
    }
}

pub fn trim_cspan(c_span: &ContainedSpan) -> ContainedSpan {
    let (cs_stoken, cs_etoken) = c_span.tokens();
    let new_s = trim(cs_stoken);
    let new_e = trim(cs_etoken);
    ContainedSpan::new(new_s, new_e)
}

pub fn trim_exp(exp: &Expression) -> Expression {
    let (mut leading_trivia, mut trailing_trivia) = exp.surrounding_trivia();
    let exp = match exp {
        Expression::Number(x) => Expression::Number(trim(x)),
        Expression::String(x) => Expression::String(trim(x)),
        Expression::Symbol(x) => Expression::Symbol(trim(x)),
        Expression::BinaryOperator { lhs, binop, rhs } => Expression::BinaryOperator {
            lhs: Box::new(trim_exp(*&lhs)),
            binop: binop.clone(),
            rhs: Box::new(trim_exp(*&rhs)),
        },
        _ => exp.clone(),
    };
    trim_token(&mut leading_trivia);
    trim_token(&mut trailing_trivia);
    exp
}

pub fn trim_prefix(prefix: &Prefix) -> Prefix {
    let new_prefix: Prefix;
    match prefix {
        Prefix::Expression(y) => {
            let x = (**y).clone();
            println!("Prefix Expression before: {:#?}", x);
            new_prefix = Prefix::Expression(Box::new(trim_exp(&x)));
        }
        Prefix::Name(y) => {
            new_prefix = Prefix::Name(trim(y));
        }
        _ => {
            new_prefix = prefix.clone();
        }
    }
    new_prefix
}

pub fn trim_suffix(minifier: &Minifier, suffix: &Suffix) -> Suffix {
    let new_suffix: Suffix;
    match suffix {
        Suffix::Call(y) => match y {
            Call::AnonymousCall(z) => {
                let new_args = minifier.minify_function_args(&z);
                new_suffix = Suffix::Call(Call::AnonymousCall(new_args));
            }
            Call::MethodCall(z) => {
                let new_args = minifier.minify_function_args(z.args());
                let new_z = z.clone().with_args(new_args);
                new_suffix = Suffix::Call(Call::MethodCall(new_z));
            }
            _ => {
                new_suffix = suffix.clone();
            }
        },
        Suffix::Index(y) => match y {
            Index::Brackets {
                brackets,
                expression,
            } => {
                let new_expression = trim_exp(expression);
                let new_y = Index::Brackets {
                    brackets: trim_cspan(brackets),
                    expression: new_expression,
                };
                new_suffix = Suffix::Index(new_y);
            }
            _ => {
                new_suffix = suffix.clone();
            }
        },
        _ => {
            new_suffix = suffix.clone();
        }
    }
    new_suffix
}

pub fn append(token_ref: &TokenReference, leading: bool, trailing: bool) -> TokenReference {
    let (mut leading_trivia, mut trailing_trivia) = token_ref.surrounding_trivia();
    let whitespace_token = Token::new(TokenType::Whitespace { characters: ShortString::new(" ") });
    if leading {
        leading_trivia.push(&whitespace_token);
    }
    if trailing {
        trailing_trivia.insert(0, &whitespace_token);
    }
    TokenReference::new(
        leading_trivia.into_iter().cloned().collect(),
        token_ref.token().clone(),
        trailing_trivia.into_iter().cloned().collect(),
    )
}
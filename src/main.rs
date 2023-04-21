use clap::Parser;
use full_moon::{
    self,
    ast::{
        punctuated::{Pair, Punctuated},
        types::TypeSpecifier,
        Assignment, Expression, LocalAssignment, Stmt, Var, Value,
    },
    tokenizer::{Token, TokenReference, TokenType}, node::Node,
};
use std::{fs::read_to_string, path::PathBuf};

/// Lua minifier
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to lua file
    #[arg(short, long)]
    file: String,
}

fn punctuate_name<T>(arr: Punctuated<T>, puncutation: &TokenReference) -> Punctuated<T> {
    let arr_len = arr.len();
    let mut new_arr: Punctuated<T> = Punctuated::new();
    let mut i = 0;
    for v in arr {
        if (i + 1) == arr_len {
            new_arr.push(Pair::new(v, None))
        } else {
            new_arr.push(Pair::Punctuated(v, puncutation.clone()))
        }
        i += 1;
    }
    new_arr
}

fn punctuate_exp<T>(arr: Punctuated<T>, puncutation: &TokenReference) -> Punctuated<T> {
    let arr_len = arr.len();
    let mut new_arr: Punctuated<T> = Punctuated::new();
    let mut i = 0;
    for v in arr {
        if (i + 1) == arr_len {
            new_arr.push(Pair::Punctuated(v, TokenReference::symbol(";").unwrap()))
        } else {
            new_arr.push(Pair::Punctuated(v, puncutation.clone()))
        }
        i += 1;
    }
    new_arr
}


fn remove_whitespace_token(token: Vec<&Token>) -> Vec<Token> {
    let mut new_token: Vec<Token> = Vec::new();
    'forin: for x in token {
        match x.token_type() {
            TokenType::Whitespace { characters: _ } => {
                continue 'forin;
            }
            _ => {
                new_token.push(x.clone())
            }
        }
    }
    new_token
}

fn remove_whitespace(token_ref: TokenReference) -> TokenReference {
    let mut leading_trivia: Vec<Token> = remove_whitespace_token(token_ref.leading_trivia().collect());
    let mut trailing_trivia: Vec<Token> = remove_whitespace_token(token_ref.trailing_trivia().collect());
    TokenReference::new(leading_trivia, token_ref.token().clone(), trailing_trivia)
}

fn remove_whitespace_value(value: Value) -> Value {
    match value {
        Value::Number(x) => {
            return Value::Number(remove_whitespace(x))
        }
        Value::String(x) => {
            return Value::String(remove_whitespace(x))
        }
        Value::Symbol(x) => {
            return Value::Symbol(remove_whitespace(x))
        }
        _ => {}
    }
    value
}   

fn remove_whitespace_exp(exp: Expression) -> Expression {
    //let (leading_trivia, trailing_trivia) = exp.surrounding_trivia();
    match exp {
        Expression::Value { value, type_assertion: _ } => {
            let new_value = remove_whitespace_value(*value);
            return Expression::Value { 
                value: Box::new(new_value),
                type_assertion: None
            }
        }
        _ => {}
    }
    exp
}

fn main() {
    println!("!!!NOT READY FOR PRODUCTION USE!!!");
    println!("Lumine is cute :3");
    let args = Args::parse();
    println!("Reading file {}...", args.file);
    let file = PathBuf::from(args.file);
    let text = read_to_string(file).expect("read input file error");
    let ast = full_moon::parse(text.as_str()).expect("parse lua script error");
    // Parse Lua block
    let block = ast.nodes();
    let mut new_stmts: Vec<(Stmt, Option<TokenReference>)> = Vec::new();
    let eq_token: Option<TokenReference> = Some(TokenReference::symbol("=").unwrap());
    // Local assignments
    let mut local_names: Punctuated<TokenReference> = Punctuated::new();
    let mut local_expressions: Punctuated<Expression> = Punctuated::new();
    let mut local_types: Vec<Option<TypeSpecifier>> = Vec::new();
    let semicolon = TokenReference::symbol(",").unwrap();
    // Assignments
    let mut global_vars: Punctuated<Var> = Punctuated::new();
    let mut global_expressions: Punctuated<Expression> = Punctuated::new();
    for stmt in block.stmts() {
        match stmt {
            Stmt::LocalAssignment(x) => {
                let name_c = x.names().clone();
                for name in name_c {
                    println!("{:#?}", name);
                    local_names.push(Pair::new(remove_whitespace(name), None));
                }
                for exp in x.expressions().clone() {
                    println!("loop");
                    local_expressions.push(Pair::new(remove_whitespace_exp(exp), None));
                }
                for var_type in x.type_specifiers() {
                    //println!("{:#?}", var_type);
                    //println!("{:#?}", var_type.cloned());
                    local_types.push(var_type.cloned());
                }
                println!("Found local assignment");
            }
            Stmt::Assignment(x) => {
                //println!("{:?}", x);
                for var in x.variables().clone() {
                    //println!("{:#?}", var.tokens());
                    match var {
                        Var::Name(ref y) => {
                            let y = Var::Name(remove_whitespace(y.clone()));
                            global_vars.push(Pair::new(y, None))
                        }
                        _ => {
                            global_vars.push(Pair::new(var, None))
                        }
                    }
                }
                for exp in x.expressions().clone() {
                    global_expressions.push(Pair::new(remove_whitespace_exp(exp), None))
                }
                println!("Found assignment");
            }
            x => {
                // TODO: remove whitespaces
                new_stmts.push((x.clone(), Some(TokenReference::symbol(";").unwrap())))
            }
        }
    }
    let local_names = punctuate_name(local_names, &semicolon);
    let local_assignments = LocalAssignment::new(local_names.clone())
        .with_names(local_names)
        //.with_type_specifiers(local_types)
        .with_equal_token(eq_token.clone())
        .with_expressions(punctuate_exp(local_expressions, &semicolon));
    let assignments = Assignment::new(
        punctuate_name(global_vars, &semicolon),
        punctuate_exp(global_expressions, &semicolon),
    )
    .with_equal_token(eq_token.unwrap());
    new_stmts.push((
        full_moon::ast::Stmt::LocalAssignment(local_assignments),
        None,
    ));
    new_stmts.push((full_moon::ast::Stmt::Assignment(assignments), None));
    //new_stmts.splice(0..0, s.iter().cloned());
    // Create a new AST
    let new_ast = ast.clone().with_nodes(block.clone().with_stmts(new_stmts));
    //println!("{:#?}", new_ast);
    println!("=== SCRIPT GENERATED ===");
    println!("-- Minified by luamine-rs\n{}", full_moon::print(&new_ast))
}

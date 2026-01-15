use clap::Parser;
use full_moon::{
    self,
    ast::{
        luau::TypeSpecifier,
        punctuated::{Pair, Punctuated},
        span::ContainedSpan,
        Assignment, Block, Call, Expression, Field, FunctionArgs, Index, LocalAssignment,
        Parameter, Prefix, Stmt, Suffix, Var,
    },
    node::Node,
    tokenizer::{Token, TokenReference, TokenType},
};
use std::{fs::read_to_string, path::PathBuf};
pub mod minifier;

/// An experimental Lua(u) minifier built using full-moon
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to lua file
    #[arg(short, long)]
    file: String,
    /// Path to the output file, if not provided, prints to stdout
    #[arg(short, long)]
    output: Option<String>,
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

fn remove_whitespace_token(token: &mut Vec<&Token>) {
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

fn remove_whitespace_leading(token_ref: &TokenReference) -> TokenReference {
    match token_ref.token().token_type() {
        TokenType::Whitespace { characters: _ } => {
            return TokenReference::symbol("").unwrap();
        }
        _ => {
            let (mut leading_trivia, mut trailing_trivia) = token_ref.surrounding_trivia();
            remove_whitespace_token(&mut leading_trivia);
            TokenReference::new(
                leading_trivia.into_iter().cloned().collect(),
                token_ref.token().clone(),
                trailing_trivia.into_iter().cloned().collect(),
            )
        }
    }
}

fn remove_whitespace(token_ref: &TokenReference) -> TokenReference {
    match token_ref.token().token_type() {
        TokenType::Whitespace { characters: _ } => {
            return TokenReference::symbol("").unwrap();
        }
        _ => {
            let (mut leading_trivia, mut trailing_trivia) = token_ref.surrounding_trivia();
            remove_whitespace_token(&mut leading_trivia);
            remove_whitespace_token(&mut trailing_trivia);
            TokenReference::new(
                leading_trivia.into_iter().cloned().collect(),
                token_ref.token().clone(),
                trailing_trivia.into_iter().cloned().collect(),
            )
        }
    }
}

fn remove_whitespace_cspan(c_span: &ContainedSpan) -> ContainedSpan {
    let (cs_stoken, cs_etoken) = c_span.tokens();
    let new_s = remove_whitespace(cs_stoken);
    let new_e = remove_whitespace(cs_etoken);
    ContainedSpan::new(new_s, new_e)
}

fn remove_whitespace_exp(exp: &Expression) -> Expression {
    let (mut leading_trivia, mut trailing_trivia) = exp.surrounding_trivia();
    let exp = match exp {
        Expression::Number(x) => Expression::Number(remove_whitespace(x)),
        Expression::String(x) => Expression::String(remove_whitespace(x)),
        Expression::Symbol(x) => Expression::Symbol(remove_whitespace(x)),
        Expression::BinaryOperator { lhs, binop, rhs } => Expression::BinaryOperator {
            lhs: Box::new(remove_whitespace_exp(*&lhs)),
            binop: binop.clone(),
            rhs: Box::new(remove_whitespace_exp(*&rhs)),
        },
        _ => {exp.clone()},
    };
    remove_whitespace_token(&mut leading_trivia);
    remove_whitespace_token(&mut trailing_trivia);
    exp
}

fn remove_whitespace_prefix(prefix: &Prefix) -> Prefix {
    let new_prefix: Prefix;
    match prefix {
        Prefix::Expression(y) => {
            let x= (**y).clone();
            println!("Prefix Expression before: {:#?}", x);
            new_prefix = Prefix::Expression(Box::new(remove_whitespace_exp(&x)));
        }
        Prefix::Name(y) => {
            new_prefix = Prefix::Name(remove_whitespace(y));
        }
        _ => {
            new_prefix = prefix.clone();
        }
    }
    new_prefix
}

fn remove_whitespace_suffix(suffix: &Suffix) -> Suffix {
    let new_suffix: Suffix;
    match suffix {
        Suffix::Call(y) => match y {
            Call::AnonymousCall(z) => {
                let new_args = minify_function_args(&z);
                new_suffix = Suffix::Call(Call::AnonymousCall(new_args));
            }
            Call::MethodCall(z) => {
                let new_args = minify_function_args(z.args());
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
                let new_expression = remove_whitespace_exp(expression);
                let new_y = Index::Brackets {
                    brackets: remove_whitespace_cspan(brackets),
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

fn minify_function_args(function_args: &FunctionArgs) -> FunctionArgs {
    let semicolon = TokenReference::symbol(",").unwrap();
    match function_args {
        FunctionArgs::Parentheses {
            parentheses,
            arguments,
        } => {
            let mut new_args: Punctuated<Expression> = Punctuated::new();
            for arg in arguments {
                let new_arg = remove_whitespace_exp(arg);
                new_args.push(Pair::new(new_arg, None))
            }
            new_args = punctuate_name(new_args, &semicolon);
            FunctionArgs::Parentheses {
                parentheses: remove_whitespace_cspan(parentheses),
                arguments: new_args,
            }
        }
        FunctionArgs::TableConstructor(x) => {
            let mut new_fields: Punctuated<Field> = Punctuated::new();
            for field in x.fields() {
                let new_field: Field;
                match field {
                    Field::ExpressionKey {
                        brackets,
                        key,
                        equal,
                        value,
                    } => {
                        let new_key = remove_whitespace_exp(key);
                        let new_value = remove_whitespace_exp(value);
                        new_field = Field::ExpressionKey {
                            brackets: remove_whitespace_cspan(brackets),
                            key: new_key,
                            equal: equal.clone(),
                            value: new_value,
                        }
                    }
                    Field::NameKey { key, equal, value } => {
                        let new_key = remove_whitespace(key);
                        let new_value = remove_whitespace_exp(value);
                        new_field = Field::NameKey {
                            key: new_key,
                            equal: equal.clone(),
                            value: new_value,
                        }
                    }
                    _ => {
                        new_field = field.clone();
                    }
                }
                new_fields.push(Pair::new(new_field, None));
            }
            new_fields = punctuate_name(new_fields, &semicolon);
            let new_x = x.clone().with_fields(new_fields);
            full_moon::ast::FunctionArgs::TableConstructor(new_x)
        }
        _ => function_args.clone(),
    }
}

fn minify_block(block: &Block) -> Block {
    let eq_token: Option<TokenReference> = Some(TokenReference::symbol("=").unwrap());
    let nil_symbol: Expression = Expression::Symbol(TokenReference::symbol("nil").unwrap());
    let mut new_stmts: Vec<(Stmt, Option<TokenReference>)> = Vec::new();
    // Local assignments`
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
                println!("Found local assignment {:?}", x.names());
                let name_c = x.names().clone();
                let exp_c = x.expressions().clone();
                let diff_len = name_c.len() - exp_c.len();
                for name in name_c {
                    //println!("{:#?}", name);
                    local_names.push(Pair::new(remove_whitespace(&name), None));
                }
                for exp in exp_c {
                    println!("loop");
                    local_expressions.push(Pair::new(remove_whitespace_exp(&exp), None));
                }
                println!("{} diff len between name value", diff_len);
                if diff_len > 0 {
                    for _ in 0..diff_len {
                        local_expressions.push(Pair::new(nil_symbol.clone(), None));
                    }
                }
                for var_type in x.type_specifiers() {
                    //println!("{:#?}", var_type);
                    //println!("{:#?}", var_type.cloned());
                    local_types.push(var_type.cloned());
                }
            }
            Stmt::Assignment(x) => {
                //println!("{:?}", x);
                //println!("Found assignment {:?}", x);
                println!("Found global var assignment {:?}", x);
                for var in x.variables().clone() {
                    //println!("{:#?}", var.tokens());
                    match var {
                        Var::Name(ref y) => {
                            let y = Var::Name(remove_whitespace(y));
                            println!("Pushing global var: {:#?}", y);
                            global_vars.push(Pair::new(y, None))
                        }
                        Var::Expression(y) => {
                            let new_prefix = remove_whitespace_prefix(y.prefix());
                            let mut new_suffixes: Vec<Suffix> = Vec::new();
                            for suffix in y.suffixes() {
                                new_suffixes.push(remove_whitespace_suffix(suffix));
                            }
                            let new_y = y
                                .clone()
                                .with_prefix(new_prefix)
                                .with_suffixes(new_suffixes);
                            global_vars.push(Pair::new(
                                full_moon::ast::Var::Expression(Box::new(new_y)),
                                None,
                            ))
                        }
                        _ => global_vars.push(Pair::new(var, None)),
                    }
                }
                for exp in x.expressions().clone() {
                    let new_exp = remove_whitespace_exp(&exp);
                    println!("Pushing global expression: {:#?}", new_exp);
                    global_expressions.push(Pair::new(new_exp, None))
                }
            }
            Stmt::LocalFunction(x) => {
                let body = x.body();
                let params_parentheses = body.parameters_parentheses();
                let new_parentheses = remove_whitespace_cspan(params_parentheses);
                let mut new_params: Punctuated<Parameter> = Punctuated::new();
                for param in body.parameters() {
                    match param {
                        Parameter::Ellipsis(x) => {
                            let new_param = Parameter::Ellipsis(remove_whitespace(x));
                            new_params.push(Pair::new(new_param, None));
                        }
                        Parameter::Name(x) => {
                            let new_param = Parameter::Ellipsis(remove_whitespace(x));
                            new_params.push(Pair::new(new_param, None));
                        }
                        _ => {}
                    }
                }
                new_params = punctuate_name(new_params, &semicolon);
                let new_local = remove_whitespace_leading(x.local_token());
                let new_end = remove_whitespace(body.end_token());
                let new_body = body
                    .clone()
                    .with_parameters_parentheses(new_parentheses)
                    .with_parameters(new_params)
                    .with_block(minify_block(body.block()))
                    .with_end_token(new_end);
                let new_x = x
                    .clone()
                    .with_function_token(remove_whitespace_leading(x.function_token()))
                    .with_local_token(new_local)
                    .with_body(new_body);
                new_stmts.push((
                    full_moon::ast::Stmt::LocalFunction(new_x),
                    Some(TokenReference::symbol(";").unwrap()),
                ))
            }
            Stmt::FunctionDeclaration(x) => {
                let body = x.body();
                let params_parentheses = body.parameters_parentheses();
                let new_parentheses = remove_whitespace_cspan(params_parentheses);
                let mut new_params: Punctuated<Parameter> = Punctuated::new();
                for param in body.parameters() {
                    match param {
                        Parameter::Ellipsis(x) => {
                            let new_param = Parameter::Ellipsis(remove_whitespace(x));
                            new_params.push(Pair::new(new_param, None));
                        }
                        Parameter::Name(x) => {
                            let new_param = Parameter::Ellipsis(remove_whitespace(x));
                            new_params.push(Pair::new(new_param, None));
                        }
                        _ => {}
                    }
                }
                new_params = punctuate_name(new_params, &semicolon);
                let new_end = remove_whitespace(body.end_token());
                let new_body = body
                    .clone()
                    .with_parameters_parentheses(new_parentheses)
                    .with_parameters(new_params)
                    .with_block(minify_block(body.block()))
                    .with_end_token(new_end);
                let new_x = x
                    .clone()
                    .with_function_token(remove_whitespace_leading(x.function_token()))
                    .with_body(new_body);
                new_stmts.push((
                    full_moon::ast::Stmt::FunctionDeclaration(new_x),
                    Some(TokenReference::symbol(";").unwrap()),
                ))
            }
            Stmt::FunctionCall(x) => {
                // println!("{:#?}", x);
                let new_prefix = remove_whitespace_prefix(x.prefix());
                let mut new_suffixes: Vec<Suffix> = Vec::new();
                for suffix in x.suffixes() {
                    new_suffixes.push(remove_whitespace_suffix(suffix));
                }
                let new_x = x
                    .clone()
                    .with_prefix(new_prefix)
                    .with_suffixes(new_suffixes);
                new_stmts.push((
                    full_moon::ast::Stmt::FunctionCall(new_x),
                    Some(TokenReference::symbol(";").unwrap()),
                ))
            }
            _ => {
                // TODO: remove whitespaces
                new_stmts.push((stmt.clone(), Some(TokenReference::symbol(";").unwrap())))
            }
        }
    }
    if local_names.len() > 0 {
        let local_names = punctuate_name(local_names, &semicolon);
        let local_assignments = LocalAssignment::new(local_names.clone())
            .with_names(local_names)
            //.with_type_specifiers(local_types)
            .with_equal_token(eq_token.clone())
            .with_expressions(punctuate_exp(local_expressions, &semicolon));
        new_stmts.splice(
            0..0,
            vec![(
                full_moon::ast::Stmt::LocalAssignment(local_assignments),
                None,
            )]
            .iter()
            .cloned(),
        );
    }
    if global_vars.len() > 0 {
        let assignments = Assignment::new(
            punctuate_name(global_vars, &semicolon),
            punctuate_exp(global_expressions, &semicolon),
        )
        .with_equal_token(eq_token.unwrap());
        new_stmts.splice(
            0..0,
            vec![(full_moon::ast::Stmt::Assignment(assignments), None)]
                .iter()
                .cloned(),
        );
    }
    block.clone().with_stmts(new_stmts)
}

fn main() {
    println!("!!! NOT READY FOR PRODUCTION USE !!!");
    println!("Lumine is cute :3");
    let args = Args::parse();
    println!("Reading file {}...", args.file);
    let file = PathBuf::from(args.file);
    let text = read_to_string(file).expect("read input file error");
    let ast = full_moon::parse(text.as_str()).expect("parse lua script error");
    // Parse Lua block
    let block = ast.nodes();
    let new_block = minify_block(block);
    // Create a new AST
    let new_ast = ast.clone().with_nodes(new_block);
    //println!("{:#?}", new_ast);
    println!("\n=== SCRIPT GENERATED ===\n");
    println!("-- Minified by luamine-rs");
    println!(
        "-- Code may not be usable, report bugs at: https://github.com/teppyboy/luamine-rs/issues"
    );
    println!("{}", &new_ast);
}

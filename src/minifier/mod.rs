use full_moon::{
    self,
    ast::{
        luau::TypeSpecifier,
        punctuated::{Pair, Punctuated},
        Assignment, Block, Expression, Field, FunctionArgs, LocalAssignment, Parameter, Stmt,
        Suffix, Var,
    },
    tokenizer::TokenReference,
};

use crate::minifier::constants::*;

mod punctuator;
mod whitespace;
mod constants;

pub(crate) struct Minifier {
    code: String,
}

impl Minifier {
    pub fn new(code: &str) -> Self {
        Minifier {
            code: String::from(code),
        }
    }

    fn minify_function_args(&self, function_args: &FunctionArgs) -> FunctionArgs {
        match function_args {
            FunctionArgs::Parentheses {
                parentheses,
                arguments,
            } => {
                let mut new_args: Punctuated<Expression> = Punctuated::new();
                for arg in arguments {
                    let new_arg = whitespace::trim_exp(arg);
                    new_args.push(Pair::new(new_arg, None))
                }
                new_args = punctuator::punctuate_name(new_args, &COMMA);
                FunctionArgs::Parentheses {
                    parentheses: whitespace::trim_cspan(parentheses),
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
                            let new_key = whitespace::trim_exp(key);
                            let new_value = whitespace::trim_exp(value);
                            new_field = Field::ExpressionKey {
                                brackets: whitespace::trim_cspan(brackets),
                                key: new_key,
                                equal: equal.clone(),
                                value: new_value,
                            }
                        }
                        Field::NameKey { key, equal, value } => {
                            let new_key = whitespace::trim(key);
                            let new_value = whitespace::trim_exp(value);
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
                new_fields = punctuator::punctuate_name(new_fields, &COMMA);
                let new_x = x.clone().with_fields(new_fields);
                full_moon::ast::FunctionArgs::TableConstructor(new_x)
            }
            _ => function_args.clone(),
        }
    }

    fn minify_block(&self, block: &Block) -> Block {
        let mut new_stmts: Vec<(Stmt, Option<TokenReference>)> = Vec::new();
        // Local assignments`
        let mut local_names: Punctuated<TokenReference> = Punctuated::new();
        let mut local_expressions: Punctuated<Expression> = Punctuated::new();
        let mut local_types: Vec<Option<TypeSpecifier>> = Vec::new();
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
                        local_names.push(Pair::new(whitespace::trim(&name), None));
                    }
                    for exp in exp_c {
                        println!("loop");
                        local_expressions.push(Pair::new(whitespace::trim_exp(&exp), None));
                    }
                    println!("{} diff len between name value", diff_len);
                    if diff_len > 0 {
                        for _ in 0..diff_len {
                            local_expressions.push(Pair::new(NIL_SYMBOL.clone(), None));
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
                                let y = Var::Name(whitespace::trim(y));
                                println!("Pushing global var: {:#?}", y);
                                global_vars.push(Pair::new(y, None))
                            }
                            Var::Expression(y) => {
                                let new_prefix = whitespace::trim_prefix(y.prefix());
                                let mut new_suffixes: Vec<Suffix> = Vec::new();
                                for suffix in y.suffixes() {
                                    new_suffixes.push(whitespace::trim_suffix(self, suffix));
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
                        let new_exp = whitespace::trim_exp(&exp);
                        println!("Pushing global expression: {:#?}", new_exp);
                        global_expressions.push(Pair::new(new_exp, None))
                    }
                }
                Stmt::LocalFunction(x) => {
                    let body = x.body();
                    let params_parentheses = body.parameters_parentheses();
                    let new_parentheses = whitespace::trim_cspan(params_parentheses);
                    let mut new_params: Punctuated<Parameter> = Punctuated::new();
                    for param in body.parameters() {
                        match param {
                            Parameter::Ellipsis(x) => {
                                let new_param = Parameter::Ellipsis(whitespace::trim(x));
                                new_params.push(Pair::new(new_param, None));
                            }
                            Parameter::Name(x) => {
                                let new_param = Parameter::Ellipsis(whitespace::trim(x));
                                new_params.push(Pair::new(new_param, None));
                            }
                            _ => {}
                        }
                    }
                    new_params = punctuator::punctuate_name(new_params, &*COMMA);
                    let new_local = whitespace::trim_leading(x.local_token());
                    let new_end = whitespace::trim(body.end_token());
                    let new_body = body
                        .clone()
                        .with_parameters_parentheses(new_parentheses)
                        .with_parameters(new_params)
                        .with_block(self.minify_block(body.block()))
                        .with_end_token(new_end);
                    let new_x = x
                        .clone()
                        .with_function_token(whitespace::append(&whitespace::trim_leading(x.function_token()), true, true))
                        .with_local_token(new_local)
                        .with_body(new_body);
                    new_stmts.push((
                        full_moon::ast::Stmt::LocalFunction(new_x),
                        SEMICOLON.clone(),
                    ))
                }
                Stmt::FunctionDeclaration(x) => {
                    let body = x.body();
                    let params_parentheses = body.parameters_parentheses();
                    let new_parentheses = whitespace::trim_cspan(params_parentheses);
                    let mut new_params: Punctuated<Parameter> = Punctuated::new();
                    for param in body.parameters() {
                        match param {
                            Parameter::Ellipsis(x) => {
                                let new_param = Parameter::Ellipsis(whitespace::trim(x));
                                new_params.push(Pair::new(new_param, None));
                            }
                            Parameter::Name(x) => {
                                let new_param = Parameter::Ellipsis(whitespace::trim(x));
                                new_params.push(Pair::new(new_param, None));
                            }
                            _ => {}
                        }
                    }
                    new_params = punctuator::punctuate_name(new_params, &COMMA);
                    let new_end = whitespace::trim(body.end_token());
                    let new_body = body
                        .clone()
                        .with_parameters_parentheses(new_parentheses)
                        .with_parameters(new_params)
                        .with_block(self.minify_block(body.block()))
                        .with_end_token(new_end);
                    let new_x = x
                        .clone()
                        .with_function_token(whitespace::append(&whitespace::trim_leading(x.function_token()), false, true))
                        .with_body(new_body);
                    new_stmts.push((
                        full_moon::ast::Stmt::FunctionDeclaration(new_x),
                        SEMICOLON.clone(),
                    ))
                }
                Stmt::FunctionCall(x) => {
                    // println!("{:#?}", x);
                    let new_prefix = whitespace::trim_prefix(x.prefix());
                    let mut new_suffixes: Vec<Suffix> = Vec::new();
                    for suffix in x.suffixes() {
                        new_suffixes.push(whitespace::trim_suffix(self, suffix));
                    }
                    let new_x = x
                        .clone()
                        .with_prefix(new_prefix)
                        .with_suffixes(new_suffixes);
                    new_stmts.push((
                        full_moon::ast::Stmt::FunctionCall(new_x),
                        SEMICOLON.clone(),
                    ))
                }
                _ => {
                    // TODO: remove whitespaces
                    new_stmts.push((stmt.clone(), SEMICOLON.clone()))
                }
            }
        }
        if local_names.len() > 0 {
            let local_names = punctuator::punctuate_name(local_names, &COMMA);
            let local_assignments = LocalAssignment::new(local_names.clone())
                .with_names(local_names)
                //.with_type_specifiers(local_types)
                .with_equal_token(EQ_TOKEN.clone())
                .with_expressions(punctuator::punctuate_exp(local_expressions, &COMMA));
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
                punctuator::punctuate_name(global_vars, &COMMA),
                punctuator::punctuate_exp(global_expressions, &COMMA),
            )
            .with_equal_token(EQ_TOKEN.as_ref().unwrap().clone());
            new_stmts.splice(
                0..0,
                vec![(full_moon::ast::Stmt::Assignment(assignments), None)]
                    .iter()
                    .cloned(),
            );
        }
        block.clone().with_stmts(new_stmts)
    }

    pub fn minify(&self) -> String {
        let ast = full_moon::parse(self.code.as_str()).expect("parse lua script error");
        let block = ast.nodes();
        let new_block = self.minify_block(block);
        let new_ast = ast.clone().with_nodes(new_block);
        format!("-- Minified by luamine-rs v{}\n-- Code may not be usable, report bugs at: https://github.com/teppyboy/luamine-rs/issues\n{}", env!("CARGO_PKG_VERSION"), &new_ast)
    }
}

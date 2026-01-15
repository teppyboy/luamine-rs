use full_moon::{
    self,
    ast::punctuated::{Pair, Punctuated},
    tokenizer::TokenReference,
};

pub fn punctuate_name<T>(arr: Punctuated<T>, puncutation: &TokenReference) -> Punctuated<T> {
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

pub fn punctuate_exp<T>(arr: Punctuated<T>, puncutation: &TokenReference) -> Punctuated<T> {
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

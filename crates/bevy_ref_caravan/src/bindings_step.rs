use proc_macro::*;
use proc_macro::token_stream::IntoIter as TokenIter;

use crate::{
    common::{collect_until_punct::*, *}, 
    construction_step::construction_step, 
    entity_step::*, 
    syntax_in::*,
};

pub(crate) enum IntoNext {
    IntoNext,
    Escape,
}

pub(crate) fn bindings_step(
    caravan: TokenIter, 
    package: TokenStream,
    exit_rule: &TokenStream,
    is_nested: bool,

    entity_clause: (EntityWildcard, Vec<TokenTree>), 
    query_clause: Vec<TokenTree>,
) -> Result<(TokenIter, TokenStream), ()> {
    let (mut caravan, bindings_clause, into_next) = match collect_until_bindings_end(caravan, Vec::new(), is_nested) {
        Ok(ok) => ok,
        Err(err) => return Err(err),
    };
    
    let mut_iter = bindings_clause.iter();
    let contains_mut = contains_mut_recursive(mut_iter);

    match is_nested {
        true => {
            let package = match construction_step(package, exit_rule, entity_clause, query_clause, bindings_clause, contains_mut) {
                Ok(ok) => ok,
                Err(err) => return Err(err),
            };

            let Some(current) = caravan.next() else {
                return Ok((caravan, package))
            };

            return entity_step_entrance(caravan, package, exit_rule, true, into_next, current)
        },
        false => { 
            let package = match construction_step(package, exit_rule, entity_clause, query_clause, bindings_clause, contains_mut) {
                Ok(ok) => ok,
                Err(err) => return Err(err),
            };

            match into_next {
                IntoNext::IntoNext => {
                    let Some(current) = caravan.next() else {
                        return Ok((caravan, package))
                    };

                    return entity_step_entrance(caravan, package, exit_rule, true, IntoNext::IntoNext, current)
                },
                IntoNext::Escape => return Ok((caravan, package)),
            }
        },
    }

}

fn collect_until_bindings_end(
    mut caravan: TokenIter, 
    mut output: Vec<TokenTree>,
    is_nested: bool,
) -> Result<(TokenIter, Vec<TokenTree>, IntoNext), ()> {
    let token = caravan.next();
    let Some(token) = token else { // Expect to be un-nested or else throw an error.
        return Ok((caravan, output, IntoNext::Escape))
    };

    let TokenTree::Punct(token) = token else { // Is Punct?
        output.push(token);
        return collect_until_bindings_end(caravan, output, is_nested) // If not, continue and add token to output.
    };

    // Is valid singular token?
    match is_nested {
        true => {
            if token == NEXT { // For nested the NEXT symbol is valid.
                return Ok((caravan, output, IntoNext::Escape))
            }
        },
        false => {
            if token == LINE_BREAK { // For un-nested the LINE_BREAK symbol is valid.
                return Ok((caravan, output, IntoNext::Escape))
            }
        },
    }


    match token.spacing() { // Is a token combo?
        Spacing::Joint => {/* Proceed */},
        Spacing::Alone => {
            output.push(TokenTree::Punct(token));
            return collect_until_bindings_end(caravan, output, is_nested) // If not, continue and add token to output.
        },
    }

    // Is INTO_NEXT punct combo?
    let (results, caravan, output) = match_one_punct_combo(INTO_NEXT.iter(), caravan, token, output);
    match results {
        PunctMatch::Matching => return Ok((caravan, output, IntoNext::IntoNext)),
        _ => {
            return collect_until_bindings_end(caravan, output, is_nested) // If not, continue. (token is already added to output because of match_one_punct_combo).
        },
    }
}
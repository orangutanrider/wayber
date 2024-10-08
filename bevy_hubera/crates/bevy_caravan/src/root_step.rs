use crate::*;
use crate::{
    syntax_in::{ENTITY_PRE_PROCESS_NOTATION, EXIT_RULE_NOTATION, LINE_BREAK},
    entity_pre_processing::entity_pre_process_decleration_step, 
    exit_rule_step::exit_rule_step,
    entity_step::entity_step_entrance, 
};

pub(crate) fn root_step(
    mut caravan: TokenIter, 
    package: TokenStream,
    mut exit_rule: ExitRule,
    mut pre_process: Option<EntityPreProcess>,
) -> Result<(TokenIter, TokenStream, ExitRule, Option<EntityPreProcess>), ()> {
    let Some(token) = caravan.next() else {
        return Ok((caravan, package, exit_rule, pre_process))
    };

    match token {
        TokenTree::Punct(punct) => { match punct.as_char() {
            LINE_BREAK => return root_step(caravan, package, exit_rule, pre_process),
            ENTITY_PRE_PROCESS_NOTATION => {
                (caravan, pre_process) = match entity_pre_process_decleration_step(caravan) {
                    Ok(ok) => ok,
                    Err(err) => return Err(err),
                };
                return root_step(caravan, package, exit_rule, pre_process)
            },
            EXIT_RULE_NOTATION => {
                let caravan = match exit_rule_step(caravan, &mut exit_rule, punct.spacing()) {
                    Ok(ok) => ok,
                    Err(err) => return Err(err),
                };
            
                return root_step(caravan, package, exit_rule, pre_process)
            },
            _ => { // Miscellaneous
                let token = TokenTree::Punct(punct);
                let (caravan, package) = match entity_step_entrance(caravan, package, &exit_rule, &pre_process, false, false, token) {
                    Ok(ok) => ok,
                    Err(err) => return Err(err),
                };
            
                return root_step(caravan, package, exit_rule, pre_process)
            },
        }},
        _ => {
            let (caravan, package) = match entity_step_entrance(caravan, package, &exit_rule, &pre_process, false, false, token) {
                Ok(ok) => ok,
                Err(err) => return Err(err),
            };
        
            return root_step(caravan, package, exit_rule, pre_process)
        },
    }
}
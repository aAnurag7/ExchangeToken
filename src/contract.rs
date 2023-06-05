use cosmwasm_std::{
    entry_point, Deps,Empty, DepsMut, Env, MessageInfo, Response, StdResult, StdError ,Addr
};

use crate::msg::{ExecuteMsg, OrderListForERC20, OrderListForERC721};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: Deps,
    _env: Env,
    _info: MessageInfo,
    msg: Empty,
) -> StdResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feat
 = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    use ExecuteMsg::*;
    match msg {
        exchange { listForseller, listForbuyer} => execute::exchange(deps, _env, _info, listForbuyer, listForseller),
    }
    Ok(Response::new())
}

mod execute {
    use super::*;

    pub fn exchange(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        listForseller: OrderListForERC20,
        listForbuyer: OrderListForERC721,
    ) {
        if checkForExchange(listForseller, listForbuyer) {
            // return Err(StdError::generic_err("Game is not open for joining"));
        }
    }

    pub fn checkForExchange(
        listForseller: OrderListForERC20,
        listForbuyer: OrderListForERC721,
    ) -> bool {
        if (listForbuyer.tokenid != listForseller.wantTokenId) {
            return false;
        }
        if listForbuyer.amountOfERC20 == listForseller.tokenAmount {
            return false;
        }

        true
    }
}

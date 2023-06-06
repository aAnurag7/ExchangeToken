
use cosmwasm_std::{
    entry_point, Empty ,to_binary, Deps, Env, MessageInfo, Response, StdError, StdResult,
};

use crate::msg::{ExecuteMsg, OrderListForERC20, OrderListForERC721};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: Deps,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> StdResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: Deps,
    _env: Env,
    _info: MessageInfo,
    signature_seller: Vec<u8>,
    signature_buyer: Vec<u8>,
    pubkey_seller: Vec<u8>,
    pubkey_buyer: Vec<u8>,
    _msg: ExecuteMsg,
) -> StdResult<Response>  {
    use ExecuteMsg::*;
    match _msg {
        Exchange { listforseller, listforbuyer} => execute::exchange(
            deps,
            signature_seller,
            signature_buyer,
            pubkey_seller,
            pubkey_buyer,
            listforbuyer,
            listforseller
        )
    }
}  

mod execute {
    use super::*;
    pub fn exchange(
        deps: Deps,
        signature_seller: Vec<u8>,
        signature_buyer: Vec<u8>,
        pubkey_seller: Vec<u8>,
        pubkey_buyer: Vec<u8>,
        listforbuyer: OrderListForERC20,
        listforseller: OrderListForERC721
    ) -> StdResult<Response> {
        let mut serialized_message = to_binary(&listforbuyer)?;
        let api = deps.api;
        let sig_verified = api.secp256k1_verify(&serialized_message, &signature_seller, &pubkey_seller)?;

        serialized_message = to_binary(&listforseller)?;
        let sig_verified_seller = api.secp256k1_verify(&serialized_message, &signature_buyer, &pubkey_buyer)?;

        if !sig_verified && !sig_verified_seller {
            return Err(StdError::generic_err("ad").into());
        }
    
        let res = Response::new()
        .add_attribute("action", "transfer")
        .add_attribute("from", listforbuyer.owner.clone())
        .add_attribute("to", listforseller.owner.clone())
        .add_attribute("amount", listforbuyer.token_amount.to_string())
        .add_attribute("action", "transfer_erc721")
        .add_attribute("sender", listforseller.owner)
        .add_attribute("recipient", listforbuyer.owner)
        .add_attribute("token_id", listforseller.tokenid.to_string());
 
        Ok(res)
    }
}

use cosmwasm_std::{
    entry_point, to_binary, Deps, DepsMut, Empty, Env, MessageInfo, Response,
    StdError, StdResult, WasmMsg, SubMsg, CosmosMsg, Binary
};

use crate::msg::ExecuteMsg;
use k256::ecdsa:: signature::Verifier;
use cw20::{Cw20ExecuteMsg};
use cw721::{Cw721ExecuteMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> StdResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    _deps: Deps,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> StdResult<Binary> {
    to_binary("")
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    use ExecuteMsg::*;
    match msg {
        Exchange {
            signature_seller,
            listforseller,
            signature_buyer,
            listforbuyer
        } => {

        let msg = listforseller.message.as_bytes();
        let rtn = listforseller.pubkey_seller.verify(msg, &signature_seller).is_ok();

        let msg_buyer = listforbuyer.message.as_bytes();
        let rtn_buyer= listforbuyer.pubkey_buyer.verify(msg_buyer, &signature_buyer).is_ok();

        if rtn != true || rtn_buyer != true {
            return Err(StdError::generic_err("\nMessage signature are incorrect"));
        }

        if listforseller.erc721_token_id != listforbuyer.erc721_token_id_want && listforbuyer.erc20_token_amount == listforseller.amount_of_erc20_want {
            return Err(StdError::generic_err("requierments of seller and buyer is not matched").into());
        }

        let msg = Cw20ExecuteMsg::TransferFrom {
            owner: listforbuyer.owner.to_string().clone(),
            recipient: listforseller.owner.to_string().clone(),
            amount: listforbuyer.erc20_token_amount.into(),
        };
        
        let msg_nft = Cw721ExecuteMsg::TransferNft {
            recipient: listforbuyer.owner.clone().into(),
            token_id: listforbuyer.erc721_token_id_want.clone().to_string(),
        };
        
        let exec: Vec<SubMsg> = vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: listforbuyer.contract_address.clone().to_string(),
                msg: to_binary(&msg).unwrap(),
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: listforseller.contract_address.to_string(),
                msg: to_binary(&msg_nft).unwrap(),
                funds: vec![],
            })),
        ];

        let res = Response::new()
        .add_attribute("action", "transfer")
        .add_attribute("from", listforbuyer.owner.clone())
        .add_attribute("to", listforseller.owner.clone())
        .add_attribute("amount", listforbuyer.erc20_token_amount.to_string())
        .add_attribute("action", "transfer_erc721")
        .add_attribute("sender", listforseller.owner)
        .add_attribute("recipient", listforbuyer.owner)
        .add_attribute("token_id", listforseller.erc721_token_id.to_string())
        .add_submessages(exec);
        
        Ok(res)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::Addr;
    use crate::msg::{OrderListForERC20, OrderListForERC721};
    use k256::ecdsa::{Signature, VerifyingKey, signature::Signer, SigningKey};
    use rand_core::OsRng;
    use ExecuteMsg::*;
    #[test]
    fn test() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let msg = String::from("message of seller");
        let msg = to_binary(&msg).unwrap();
        let signing_key = SigningKey::random(&mut OsRng);
        let signature: Signature = signing_key.sign(msg.to_string().as_bytes());
        let verify_key = VerifyingKey::from(&signing_key);

        let msg_buyer = String::from("message of seller");
        let msg_buyer = to_binary(&msg_buyer).unwrap();
        let signing_key_buyer = SigningKey::random(&mut OsRng);
        let signature_buyer: Signature = signing_key_buyer.sign(msg_buyer.to_string().as_bytes());
        let verify_key_buyer = VerifyingKey::from(&signing_key_buyer);

        let listforseller =OrderListForERC721 {
            pubkey_seller: verify_key,
            message: msg.to_string(),
            owner: Addr::unchecked("seller"),
            contract_address: Addr::unchecked("contract_erc721"),
            erc721_token_id: 2,
            amount_of_erc20_want: 200
        };

        let listforbuyer = OrderListForERC20 { pubkey_buyer: verify_key_buyer, message:  msg_buyer.to_string(), owner: Addr::unchecked("buyer"), contract_address: Addr::unchecked("contarct_erc20"), erc20_token_amount: 200, erc721_token_id_want: 2 };

        let msg = Exchange {
            signature_seller: signature, 
            listforseller:listforseller.clone(), 
            signature_buyer: signature_buyer,
            listforbuyer: listforbuyer.clone()
        };

        let res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            msg.clone(),
        ).unwrap();

        let msg = Cw20ExecuteMsg::TransferFrom {
            owner: listforbuyer.owner.to_string().clone(),
            recipient: listforseller.owner.to_string().clone(),
            amount: listforbuyer.erc20_token_amount.into(),
        };
        
        let msg_nft = Cw721ExecuteMsg::TransferNft {
            recipient: listforbuyer.owner.clone().into(),
            token_id: listforbuyer.erc721_token_id_want.clone().to_string(),
        };
        
        let exec: Vec<SubMsg> = vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: listforbuyer.contract_address.clone().to_string(),
                msg: to_binary(&msg).unwrap(),
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: listforseller.contract_address.to_string(),
                msg: to_binary(&msg_nft).unwrap(),
                funds: vec![],
            })),
        ];

        let res_check = Response::new()
        .add_attribute("action", "transfer")
        .add_attribute("from", listforbuyer.owner.clone())
        .add_attribute("to", listforseller.owner.clone())
        .add_attribute("amount", listforbuyer.erc20_token_amount.to_string())
        .add_attribute("action", "transfer_erc721")
        .add_attribute("sender", listforseller.owner)
        .add_attribute("recipient", listforbuyer.owner)
        .add_attribute("token_id", listforseller.erc721_token_id.to_string())
        .add_submessages(exec);

        assert_eq!(res, res_check);
    }
}


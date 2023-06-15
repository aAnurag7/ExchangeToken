use crate::msg::ExchangeTokenStruct;
use crate::state::LIST;
use cosmwasm_std::{to_binary, CosmosMsg, DepsMut, Response, StdResult, SubMsg, WasmMsg
};
use cw20::Cw20ExecuteMsg;
use cw721::Cw721ExecuteMsg;

pub fn exchange(deps:DepsMut, list: ExchangeTokenStruct) -> StdResult<Response> {
    let msg = Cw20ExecuteMsg::TransferFrom {
        owner: list.erc20_token_sender.to_string().clone(),
        recipient: list.erc20_token_reciever.to_string().clone(),
        amount: list.erc20_amount.into(),
    };
    
    let msg_nft = Cw721ExecuteMsg::TransferNft {
        recipient: list.erc721_token_reciever.clone().into(),
        token_id: list.token_id.clone().to_string(),
    };
    
    let exec: Vec<SubMsg> = vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: list.erc20_contract_address.clone().to_string(),
            msg: to_binary(&msg).unwrap(),
            funds: vec![],
        })),
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: list.erc721_contract_address.to_string(),
            msg: to_binary(&msg_nft).unwrap(),
            funds: vec![],
        })),
    ];
    
    LIST.remove(
        deps.storage,
        (
            list.token_id,
            list.erc721_contract_address,
        ),
    );
    
    let res = Response::new()
        .add_attribute("action", "transfer")
        .add_attribute("from", list.erc20_token_sender.clone())
        .add_attribute("to", list.erc20_token_reciever.clone())
        .add_attribute("amount", list.erc20_amount.to_string())
        .add_attribute("action", "transfer_erc721")
        .add_attribute("sender", list.erc721_token_sender)
        .add_attribute("recipient", list.erc721_token_reciever)
        .add_attribute("token_id", list.token_id.to_string())
        .add_submessages(exec);
    
    Ok(res)
}

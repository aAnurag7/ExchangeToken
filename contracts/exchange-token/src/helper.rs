use crate::msg::{OrderListForERC20, OrderListForERC721};
use crate::state::LIST;
use cosmwasm_std::{to_binary, CosmosMsg, DepsMut, Response, StdResult, SubMsg, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use cw721::Cw721ExecuteMsg;

pub fn exchange(deps:DepsMut, list_for_buyer: OrderListForERC20, curr_list: OrderListForERC721, current_erc20_amount: u64) -> StdResult<Response> {
    let msg = Cw20ExecuteMsg::TransferFrom {
        owner: list_for_buyer.owner.to_string().clone(),
        recipient: curr_list.owner.to_string().clone(),
        amount: current_erc20_amount.into(),
    };
    
    let msg_nft = Cw721ExecuteMsg::TransferNft {
        recipient: list_for_buyer.owner.clone().into(),
        token_id: list_for_buyer.erc721_token_id_want.clone().to_string(),
    };
    
    let exec: Vec<SubMsg> = vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: list_for_buyer.contract_address.clone().to_string(),
            msg: to_binary(&msg).unwrap(),
            funds: vec![],
        })),
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: curr_list.contract_address.to_string(),
            msg: to_binary(&msg_nft).unwrap(),
            funds: vec![],
        })),
    ];
    
    LIST.remove(
        deps.storage,
        (
            list_for_buyer.erc721_token_id_want,
            list_for_buyer.erc721_contract_address,
        ),
    );
    
    let res = Response::new()
        .add_attribute("action", "transfer")
        .add_attribute("from", list_for_buyer.owner.clone())
        .add_attribute("to", curr_list.owner.clone())
        .add_attribute("amount", current_erc20_amount.to_string())
        .add_attribute("action", "transfer_erc721")
        .add_attribute("sender", curr_list.owner)
        .add_attribute("recipient", list_for_buyer.owner)
        .add_attribute("token_id", curr_list.erc721_token_id.to_string())
        .add_submessages(exec);
    
    Ok(res)
}

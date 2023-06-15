use crate::msg::{ExecuteMsg, OrderListForERC20, OrderListForERC721, QueryMsg};
use crate::helper;
use crate::state::LIST;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo,
    Response, StdError, StdResult
};
use crate::auction::auction;


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> StdResult<Response> {
    Ok(Response::new())
}

#[allow(dead_code)]
pub fn query(deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;
    match _msg {
        OrderList{ token_id, contract_address} => to_binary(&query::order_list(deps, token_id, contract_address)?),
    }
}

mod query {
    use super::*;
    use cosmwasm_std::Addr;
    pub fn order_list(deps: Deps, id:u64, contract_address: Addr ) -> StdResult<OrderListForERC721>{
        let list = LIST.load(deps.storage, (id, contract_address))?;
        Ok(list)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    use ExecuteMsg::*;
    match msg {
        Register { list_for_seller } => execute::register(deps, list_for_seller),
        Exchange { list_for_buyer } => execute::exchange(deps, list_for_buyer),
        EnglishAuction { list_for_buyer } => auction::english_auction(deps, env, list_for_buyer),
        ExchangeEnglishBid { list_for_buyer } => auction::english_bid_exchange(deps, info, env, list_for_buyer),
        DutchExchange { list_for_buyer } => auction::dutch_auction_exchange(deps, env, list_for_buyer),
        Clean {list_for_seller} => auction::clean(deps, env,  list_for_seller),
    }
}

mod execute {
    use super::*;

    pub fn register(
        deps: DepsMut,
        list_for_seller: OrderListForERC721,
    ) -> StdResult<Response> {

        LIST.update(
            deps.storage,
            (
                list_for_seller.erc721_token_id.clone(),
                list_for_seller.contract_address.clone(),
            ),
            |res| {
                if res.is_some() {
                    return Err(StdError::generic_err("token is already present"));
                }

                Ok(list_for_seller)
            }
        )?;

        Ok(Response::new())   
    }

    pub fn exchange(deps: DepsMut, list_for_buyer: OrderListForERC20) -> StdResult<Response> {
        let curr_list = LIST.load(
            deps.storage,
            (
                list_for_buyer.erc721_token_id_want.clone(),
                list_for_buyer.erc721_contract_address.clone(),
            ),
        )?;

        if curr_list.highest_bid != list_for_buyer.amount_of_erc20 {
            return Err(StdError::generic_err("ERC20 token amount not matched"));
        }

        helper::exchange(deps, list_for_buyer.clone(), curr_list, list_for_buyer.amount_of_erc20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::ExecuteMsg;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{StdError, Addr, SubMsg, WasmMsg};
    use cw20::Cw20ExecuteMsg;
    use cw721::Cw721ExecuteMsg;
    use ExecuteMsg::*;
    use QueryMsg::*;
    #[test]
    fn register_execute() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info("sender", &[]),
            Empty {},
        )
        .unwrap();

        let list = OrderListForERC721 {
            owner: Addr::unchecked("owner"),
            contract_address: Addr::unchecked("contract_erc721"),
            erc721_token_id: 2,
            highest_bid: 200,
            end_time: env.block.height+ 2,
            start_time: env.block.height,
            highest_bidder: Addr::unchecked(""),
            erc20_amount_after_time: 0,
            dutch_auction: false
        };
        
        let msg = Register {
            list_for_seller: list,
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            msg.clone(),
        );
        assert_eq!(res, Ok(Response::new()));
        
        let res = execute(deps.as_mut(), env.clone(), mock_info("owner", &[]), msg);
        assert_eq!(res, Err(StdError::generic_err("token is already present")));

        let msg = OrderList{ token_id: 2, contract_address: Addr::unchecked("contract_erc721")};
        let result = query(deps.as_ref(), mock_env(), msg).unwrap();
        let a = OrderListForERC721{
            owner: Addr::unchecked("owner")  ,
            contract_address: Addr::unchecked("contract_erc721"),
            erc721_token_id:2,
            highest_bid: 200,
            end_time: env.block.height + 2,
            start_time: env.block.height,
            highest_bidder: Addr::unchecked(""),
            erc20_amount_after_time: 0,
            dutch_auction: false
        };
        assert_eq!(result,to_binary(&a).unwrap());
    }

    #[test]
    fn execute_exchange() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let list = OrderListForERC721 {
            owner: Addr::unchecked("seller"),
            contract_address: Addr::unchecked("contract_erc721"),
            erc721_token_id: 2,
            highest_bid: 200,
            end_time: env.block.height,
            start_time: env.block.height,
            highest_bidder: Addr::unchecked(""),
            erc20_amount_after_time: 0,
            dutch_auction: false
        };
        let msg = Register {
            list_for_seller: list.clone(),
        };

        let res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("seller", &[]),
            msg.clone(),
        );
        assert_eq!(res, Ok(Response::new()));

        let list_for_buyer: OrderListForERC20 = OrderListForERC20 {
            owner: Addr::unchecked("buyer"),
            contract_address: Addr::unchecked("ERC20contract"),
            amount_of_erc20: 200,
            erc721_token_id_want: 2,
            erc721_contract_address: Addr::unchecked("contract_erc721")
        };

        let msg = Exchange {
            list_for_buyer: list_for_buyer.clone(),
        };
        let resp = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("buyer", &[]),
            msg.clone(),
        );

        let msg = Cw20ExecuteMsg::TransferFrom {
            owner: list_for_buyer.owner.to_string().clone(),
            recipient: list.owner.to_string().clone(),
            amount: list_for_buyer.amount_of_erc20.into(),
        };

        let msg_nft = Cw721ExecuteMsg::TransferNft {
            recipient: list_for_buyer.owner.clone().into(),
            token_id: list_for_buyer.erc721_token_id_want.clone().to_string(),
        };

        let exec: Vec<SubMsg> = vec![
            SubMsg::new(WasmMsg::Execute {
                contract_addr: list_for_buyer.contract_address.clone().to_string(),
                msg: to_binary(&msg).unwrap(),
                funds: vec![],
            }),
            SubMsg::new(WasmMsg::Execute {
                contract_addr: list.contract_address.to_string(),
                msg: to_binary(&msg_nft).unwrap(),
                funds: vec![],
            }),
        ];

        let res_func = Response::new()
            .add_attribute("action", "transfer")
            .add_attribute("from", list_for_buyer.owner.clone())
            .add_attribute("to", list.owner.clone())
            .add_attribute("amount", list_for_buyer.amount_of_erc20.to_string())
            .add_attribute("action", "transfer_erc721")
            .add_attribute("sender", list.owner)
            .add_attribute("recipient", list_for_buyer.owner)
            .add_attribute("token_id", list.erc721_token_id.to_string())
            .add_submessages(exec);

            assert_eq!(resp.unwrap(),res_func);

            let msg = OrderList{ token_id: 2, contract_address: Addr::unchecked("contract_erc721")};
            let result = query(deps.as_ref(), mock_env(), msg).unwrap_err();
            assert_eq!(result, StdError::NotFound { kind: "exchange_token::msg::OrderListForERC721".to_string() });
    }
}

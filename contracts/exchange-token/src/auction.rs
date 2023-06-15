use crate::msg::{OrderListForERC20, OrderListForERC721};
use crate::helper::exchange;
use crate::state::LIST;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response,
    StdError, StdResult,
};

pub mod auction {
    use super::*;
    
    pub fn dutch_auction_exchange(
        deps: DepsMut,
        env: Env,
        list_for_buyer: OrderListForERC20,
    ) -> StdResult<Response> {
        let curr_list = LIST.load(
            deps.storage,
            (
                list_for_buyer.erc721_token_id_want.clone(),
                list_for_buyer.erc721_contract_address.clone(),
            ),
        )?;

        if !curr_list.dutch_auction {
            return Err(StdError::generic_err("dutch_exchange should be true"));
        }

        if curr_list.end_time < env.block.height {
            return Err(StdError::generic_err("auction has over"));
        }

        let mut current_erc20_amount = (curr_list.highest_bid - curr_list.erc20_amount_after_time)/(curr_list.end_time - curr_list.start_time);

        current_erc20_amount = current_erc20_amount*(env.block.height - curr_list.start_time);

        current_erc20_amount = curr_list.highest_bid - current_erc20_amount;

        if list_for_buyer.amount_of_erc20 < current_erc20_amount {
            return Err(StdError::generic_err("erc20 token amount is low"));
        }

        exchange(deps, list_for_buyer, curr_list, current_erc20_amount)
    }

    pub fn english_auction(
        deps: DepsMut,
        env: Env,
        list_for_buyer: OrderListForERC20,
    ) -> StdResult<Response> {
        LIST.update(
            deps.storage,
            (
                list_for_buyer.erc721_token_id_want.clone(),
                list_for_buyer.erc721_contract_address.clone(),
            ),
            |res| {
                if res.is_some() {
                    if res.clone().unwrap().highest_bid >= list_for_buyer.amount_of_erc20 {
                        return Err(StdError::generic_err(
                            "ERC20 amount must higher than highest bid",
                        ));
                    }

                    if res.clone().unwrap().end_time < env.block.height {
                        return Err(StdError::generic_err("auction has ended"));
                    }
                }

                Ok(OrderListForERC721 {
                    highest_bid: list_for_buyer.amount_of_erc20,
                    highest_bidder: list_for_buyer.owner,
                    ..res.unwrap()
                })
            },
        )?;

        Ok(Response::new())
    }

    pub fn english_bid_exchange(
        deps: DepsMut,
        info: MessageInfo,
        env: Env,
        list_for_buyer: OrderListForERC20,
    ) -> StdResult<Response> {
        let curr_list = LIST.load(
            deps.storage,
            (
                list_for_buyer.erc721_token_id_want.clone(),
                list_for_buyer.erc721_contract_address.clone(),
            ),
        )?;

        if info.sender != curr_list.highest_bidder {
            return Err(StdError::generic_err("not highest bidder"));
        }

        if curr_list.end_time >= env.block.height {
            return Err(StdError::generic_err("auction is not over"));
        }

        exchange(deps, list_for_buyer, curr_list.clone(), curr_list.highest_bid)
    }

    pub fn clean(deps: DepsMut, env: Env, list_for_seller: OrderListForERC721) -> StdResult<Response> {
        let curr_list = LIST.load(deps.storage, (list_for_seller.erc721_token_id, list_for_seller.contract_address))?;

        if curr_list.end_time >= env.block.height {
            return Err(StdError::generic_err("auction is not over yet" ));
        }

        if !curr_list.dutch_auction {
            return Err(StdError::generic_err("token must be in dutch token list" ));
        }

        LIST.remove(deps.storage, (curr_list.erc721_token_id, curr_list.contract_address));

        Ok(Response::new())
    }
}

#[cfg(test)]
mod auction_test {
    use super::*;
    use crate::contract::*;
    use crate::msg::{ExecuteMsg::*, QueryMsg::*};
    use cosmwasm_std::{testing::{mock_dependencies, mock_env, mock_info}, Addr, Empty, to_binary, CosmosMsg ,SubMsg, WasmMsg, Uint128};
    use cw20::Cw20ExecuteMsg;
    use cw721::Cw721ExecuteMsg;
    #[test]
    fn english_auction() {
            let mut deps = mock_dependencies();
            let env = mock_env();
    
            instantiate (
                deps.as_mut(),
                env.clone(),
                mock_info("owner", &[]),
                Empty {},
            )
            .unwrap();
    
            let list = OrderListForERC721 {
                owner: Addr::unchecked("seller"),
                contract_address: Addr::unchecked("contract_erc721"),
                erc721_token_id: 2,
                highest_bid: 200,
                end_time: env.block.height + 2,
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
                mock_info("seller", &[]),
                msg.clone(),
            );
            assert_eq!(res, Ok(Response::new()));
    
            let msg = OrderList{ token_id: 2, contract_address: Addr::unchecked("contract_erc721")};
            let result = query(deps.as_ref(), mock_env(), msg).unwrap();
            let a = OrderListForERC721{
                owner: Addr::unchecked("seller")  ,
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

            let msg = EnglishAuction { 
                list_for_buyer: OrderListForERC20 { 
                    owner: Addr::unchecked("buyer"), 
                    contract_address: Addr::unchecked("contract_erc20"), 
                    amount_of_erc20: 250, 
                    erc721_token_id_want: 2, 
                    erc721_contract_address: Addr::unchecked("contract_erc721") 
                }
            };

            let res = execute(
                deps.as_mut(),
                env.clone(),
                mock_info("buyer", &[]),
                msg.clone(),
            );
            assert_eq!(res, Ok(Response::new()));

            let msg = OrderList{ token_id: 2, contract_address: Addr::unchecked("contract_erc721")};

            let result = query(deps.as_ref(), mock_env(), msg).unwrap();
            let a = OrderListForERC721{
                owner: Addr::unchecked("seller")  ,
                contract_address: Addr::unchecked("contract_erc721"),
                erc721_token_id:2,
                highest_bid: 250,
                end_time: env.block.height + 2,
                start_time: env.block.height,
                highest_bidder: Addr::unchecked("buyer"),
                erc20_amount_after_time: 0,
                dutch_auction: false
            };
            assert_eq!(result,to_binary(&a).unwrap());
    }

    #[test]
    fn english_bid_exchange() {
        let mut deps = mock_dependencies();
            let env = mock_env();
    
            instantiate (
                deps.as_mut(),
                env.clone(),
                mock_info("owner", &[]),
                Empty {},
            )
            .unwrap();
    
            let list = OrderListForERC721 {
                owner: Addr::unchecked("seller"),
                contract_address: Addr::unchecked("contract_erc721"),
                erc721_token_id: 2,
                highest_bid: 200,
                end_time: env.block.height + 2,
                start_time: env.block.height,
                highest_bidder: Addr::unchecked(""),
                erc20_amount_after_time: 0,
                dutch_auction: false
            };
            
            let msg = Register {
                list_for_seller: list.clone(),
            };
            let _res = execute(
                deps.as_mut(),
                env.clone(),
                mock_info("seller", &[]),
                msg.clone(),
            );

            let msg = EnglishAuction { 
                list_for_buyer: OrderListForERC20 { 
                    owner: Addr::unchecked("buyer"), 
                    contract_address: Addr::unchecked("contract_erc20"), 
                    amount_of_erc20: 250, 
                    erc721_token_id_want: 2, 
                    erc721_contract_address: Addr::unchecked("contract_erc721") 
                }
            };

            let _res = execute(
                deps.as_mut(),
                env.clone(),
                mock_info("buyer", &[]),
                msg.clone(),
            );

            let list_for_buyer = OrderListForERC20 { 
                owner: Addr::unchecked("buyer"), 
                contract_address: Addr::unchecked("contract_erc20"), 
                amount_of_erc20: 250, 
                erc721_token_id_want: 2, 
                erc721_contract_address: Addr::unchecked("contract_erc721") 
            };

            let msg = ExchangeEnglishBid { 
                list_for_buyer: list_for_buyer.clone()
            };
            let mock_env = mock_env();
            let initial_block_height = mock_env.block.height;
            
            let future_block_height = initial_block_height + 3; 
            
            let mut env = Env {
                block: mock_env.block.clone(),
                contract: mock_env.contract.clone(),
                transaction: None
            };
            env.block.height = future_block_height;
            
            let res_exchange_bid_english = execute(
                deps.as_mut(),
                env.clone(),
                mock_info("buyer", &[]),
                msg.clone(),
            ).unwrap();

            let msg = Cw20ExecuteMsg::TransferFrom {
                owner: list_for_buyer.owner.to_string().clone(),
                recipient: list.owner.to_string().clone(),
                amount: Uint128::new(250),
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
                    contract_addr: list.contract_address.to_string(),
                    msg: to_binary(&msg_nft).unwrap(),
                    funds: vec![],
                })),
            ];
            
            let res = Response::new()
                .add_attribute("action", "transfer")
                .add_attribute("from", list_for_buyer.owner.clone())
                .add_attribute("to", list.owner.clone())
                .add_attribute("amount",  250.to_string())
                .add_attribute("action", "transfer_erc721")
                .add_attribute("sender", list.owner)
                .add_attribute("recipient", list_for_buyer.owner)
                .add_attribute("token_id", list.erc721_token_id.to_string())
                .add_submessages(exec);
            
            assert_eq!(res_exchange_bid_english, res);
            
    }

    #[test]
    fn dutch_auction_exchange() {
            let mut deps = mock_dependencies();
            let env = mock_env();
            instantiate (
                deps.as_mut(),
                env.clone(),
                mock_info("owner", &[]),
                Empty {},
            )
            .unwrap();
    
            let list = OrderListForERC721 {
                owner: Addr::unchecked("seller"),
                contract_address: Addr::unchecked("contract_erc721"),
                erc721_token_id: 2,
                highest_bid: 200,
                end_time: env.block.height + 2,
                start_time: env.block.height,
                highest_bidder: Addr::unchecked(""),
                erc20_amount_after_time: 60,
                dutch_auction: true
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
    
            let msg = OrderList{ token_id: 2, contract_address: Addr::unchecked("contract_erc721")};
            let result = query(deps.as_ref(), mock_env(), msg).unwrap();
            assert_eq!(result,to_binary(&list.clone()).unwrap());

            let list_for_buyer = OrderListForERC20 { 
                owner: Addr::unchecked("buyer"), 
                contract_address: Addr::unchecked("contract_erc20"), 
                amount_of_erc20: 200, 
                erc721_token_id_want: 2, 
                erc721_contract_address: Addr::unchecked("contract_erc721") 
            };

            let msg = DutchExchange {                 
                list_for_buyer: list_for_buyer.clone()
            };

            let res = execute(
                deps.as_mut(),
                env.clone(),
                mock_info("buyer", &[]),
                msg.clone(),
            ).unwrap();

            let msg = Cw20ExecuteMsg::TransferFrom {
                owner: list_for_buyer.owner.to_string().clone(),
                recipient: list.owner.to_string().clone(),
                amount: list_for_buyer.amount_of_erc20.clone().into(),
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
                    contract_addr: list.contract_address.to_string().clone(),
                    msg: to_binary(&msg_nft).unwrap(),
                    funds: vec![],
                }),
            ];

            let res_func = Response::new()
            .add_attribute("action", "transfer")
            .add_attribute("from", list_for_buyer.owner.clone())
            .add_attribute("to", list.owner.clone())
            .add_attribute("amount", list_for_buyer.amount_of_erc20.to_string().  clone())
            .add_attribute("action", "transfer_erc721")
            .add_attribute("sender", list.owner)
            .add_attribute("recipient", list_for_buyer.owner.clone())
            .add_attribute("token_id", list_for_buyer.erc721_token_id_want.to_string())
            .add_submessages(exec);
            assert_eq!(res , res_func);
    }

    #[test]
    fn clean() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        instantiate (
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            Empty {},
        )
        .unwrap();

        let list = OrderListForERC721 {
            owner: Addr::unchecked("seller"),
            contract_address: Addr::unchecked("contract_erc721"),
            erc721_token_id: 2,
            highest_bid: 200,
            end_time: env.block.height - 1,
            start_time: env.block.height,
            highest_bidder: Addr::unchecked(""),
            erc20_amount_after_time: 60,
            dutch_auction: true
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

        let msg = OrderList{ token_id: 2, contract_address: Addr::unchecked("contract_erc721")};
        let result = query(deps.as_ref(), mock_env(), msg).unwrap();
        assert_eq!(result,to_binary(&list.clone()).unwrap());

        let msg = Clean { list_for_seller: list.clone() };
        let res = execute(deps.as_mut(), env, mock_info("owner", &[]), msg);
        assert_eq!(res, Ok(Response::new()));

        let msg = OrderList{ token_id: 2, contract_address: Addr::unchecked("contract_erc721")};
        let result = query(deps.as_ref(), mock_env(), msg).unwrap_err();
        assert_eq!(result, StdError::NotFound { kind: "exchange_token::msg::OrderListForERC721".to_string() });
    }
}
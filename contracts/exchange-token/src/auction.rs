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

        if curr_list.time < env.block.time {
            return Err(StdError::generic_err("bid has over"));
        }

        let mut current_erc20_amount = (curr_list.highest_bid - curr_list.erc20_amount_after_time)/curr_list.time.seconds();

        current_erc20_amount = current_erc20_amount*env.block.time.seconds();

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

                    if res.clone().unwrap().time < env.block.time {
                        return Err(StdError::generic_err("bid has ended"));
                    }
                }

                Ok(OrderListForERC721 {
                    highest_bid: list_for_buyer.amount_of_erc20,
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

        if curr_list.time >= env.block.time {
            return Err(StdError::generic_err("bid is not over"));
        }

        exchange(deps, list_for_buyer, curr_list.clone(), curr_list.highest_bid)
    }

    pub fn clean(deps: DepsMut, env: Env, list_for_seller: OrderListForERC721) -> StdResult<Response> {
        let curr_list = LIST.load(deps.storage, (list_for_seller.erc721_token_id, list_for_seller.contract_address))?;

        if curr_list.time >= env.block.time {
            return Err(StdError::generic_err("bid is not over yet" ));
        }

        if !curr_list.dutch_auction {
            return Err(StdError::generic_err("token must be in dutch token list" ));
        }

        LIST.remove(deps.storage, (curr_list.erc721_token_id, curr_list.contract_address));

        Ok(Response::new())
    }
}

use crate::msg::{ExecuteMsg, OrderListForERC20, OrderListForERC721};
use crate::state::LIST;
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Empty, Env, MessageInfo,
    Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use cw721::Cw721ExecuteMsg;

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
pub fn query(_deps: Deps, _env: Env, _msg: Empty) -> StdResult<Binary> {
    to_binary("")
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> StdResult<Response> {
    use ExecuteMsg::*;
    match _msg {
        Register { list_for_seller } => execute::register(deps, _info, list_for_seller),
        Exchange { list_for_buyer } => execute::exchange(deps, list_for_buyer),
    }
}

mod execute {
    use super::*;

    pub fn register(
        deps: DepsMut,
        info: MessageInfo,
        list_for_seller: OrderListForERC721,
    ) -> StdResult<Response> {
        let curr = LIST.load(
            deps.storage,
            (
                list_for_seller.erc721_token_id.clone(),
                list_for_seller.contract_address.clone(),
            ),
        );

        match curr {
            Ok(_) => {
                return Err(StdError::generic_err("token is already present"));
            }

            Err(_) => {
                LIST.save(
                    deps.storage,
                    (
                        list_for_seller.erc721_token_id.clone(),
                        list_for_seller.contract_address.clone(),
                    ),
                    &list_for_seller,
                )?;
            }
        }
        Ok(Response::new())
    }

    pub fn exchange(deps: DepsMut, list_for_buyer: OrderListForERC20) -> StdResult<Response> {
        let curr_list = LIST.load(
            deps.storage,
            (
                list_for_buyer.erc721_token_id_want.clone(),
                list_for_buyer.contract_address.clone(),
            ),
        )?;

        if curr_list.amount_of_erc20_want != list_for_buyer.amount_of_erc20 {
            return Err(StdError::generic_err("ERC20 token amount not matched"));
        }

        let msg = Cw20ExecuteMsg::TransferFrom {
            owner: list_for_buyer.owner.to_string().clone(),
            recipient: curr_list.owner.to_string().clone(),
            amount: list_for_buyer.amount_of_erc20.into(),
        };

        let msg_nft = Cw721ExecuteMsg::TransferNft {
            recipient: list_for_buyer.owner.clone().into(),
            token_id: list_for_buyer.erc721_token_id_want.clone().to_string(),
        };

        let exec: Vec<SubMsg> = vec![
            SubMsg::new(WasmMsg::Execute {
                contract_addr: list_for_buyer.contract_address.clone().to_string(),
                msg: to_binary(&msg)?,
                funds: vec![],
            }),
            SubMsg::new(WasmMsg::Execute {
                contract_addr: curr_list.contract_address.to_string(),
                msg: to_binary(&msg_nft)?,
                funds: vec![],
            }),
        ];

        LIST.remove(deps.storage, (list_for_buyer.erc721_token_id_want, list_for_buyer.contract_address));

        let res = Response::new()
            .add_attribute("action", "transfer")
            .add_attribute("from", list_for_buyer.owner.clone())
            .add_attribute("to", curr_list.owner.clone())
            .add_attribute("amount", list_for_buyer.amount_of_erc20.to_string())
            .add_attribute("action", "transfer_erc721")
            .add_attribute("sender", curr_list.owner)
            .add_attribute("recipient", list_for_buyer.owner)
            .add_attribute("token_id", curr_list.erc721_token_id.to_string())
            .add_submessages(exec);

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::ExecuteMsg;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::StdError;
    use cw_multi_test::{App, ContractWrapper, Executor};
    use ExecuteMsg::*;

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
            contract_address: Addr::unchecked("contract"),
            erc721_token_id: 2,
            amount_of_erc20_want: 200,
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

        let res = execute(deps.as_mut(), env, mock_info("owner", &[]), msg);
        assert_eq!(res, Err(StdError::generic_err("token is already present")));
    }

    #[test]
    fn execute_exchange() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let list = OrderListForERC721 {
            owner: Addr::unchecked("owner"),
            contract_address: Addr::unchecked("contract"),
            erc721_token_id: 2,
            amount_of_erc20_want: 200,
        };
        let msg = Register {
            list_for_seller: list.clone(),
        };

        let res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            msg.clone(),
        );
        assert_eq!(res, Ok(Response::new()));

        let list_for_buyer: OrderListForERC20 = OrderListForERC20 {
            owner: Addr::unchecked("buyer"),
            contract_address: Addr::unchecked("ERC271contract"),
            amount_of_erc20: 200,
            erc721_token_id_want: 2,
        };

        let msg = Exchange {
            list_for_buyer: list_for_buyer.clone(),
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("buyer", &[]),
            msg.clone(),
        );
        let responsexpected = Response::new()
            .add_attribute("action", "transfer")
            .add_attribute("from", list_for_buyer.owner.clone())
            .add_attribute("to", list.owner.clone())
            .add_attribute("amount", list_for_buyer.amount_of_erc20.to_string())
            .add_attribute("action", "transfer_erc721")
            .add_attribute("sender", list.owner)
            .add_attribute("recipient", list_for_buyer.owner)
            .add_attribute("token_id", list_for_buyer.erc721_token_id_want.to_string());
        println!("{:?}", res);
        assert_eq!(res.unwrap(), responsexpected);
    }

    #[test]
    fn multitest() {
        let mut app = App::default();
        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &Empty {},
                &[],
                "Contract",
                None,
            )
            .unwrap();

        WasmMsg::Execute {
            contract_addr: "token_contract_address".into(), // Replace with the actual token contract address
            msg: to_binary(&Cw20ExecuteMsg::IncreaseAllowance {
                spender: "seller".into(),
                amount: Uint128::new(200),
                expires: None,
            })
            .unwrap(),
            funds: vec![],
        };

        let list = OrderListForERC721 {
            owner: Addr::unchecked("seller"),
            contract_address: Addr::unchecked("token_contract_address"),
            erc721_token_id: 2,
            amount_of_erc20_want: 200,
        };
        let msg = Register {
            list_for_seller: list,
        };

        let resp = app.execute_contract(Addr::unchecked("owner"), addr.clone(), &msg, &[]);

        let token_msg = WasmMsg::Execute {
            contract_addr: "token_contract_address_s".into(), // Replace with the actual token contract address
            msg: to_binary(&Cw721ExecuteMsg::Approve {
                spender: addr.to_string(),
                token_id: 2.to_string(),
            })
            .unwrap(),
            funds: vec![],
        };

        let list = OrderListForERC20 {
            owner: Addr::unchecked("seller"),
            contract_address: Addr::unchecked("token_contract_address_2"),
            amount_of_erc20: 200,
            erc721_token_id_want: 2,
        };
        let msg = Exchange {
            list_for_buyer: list,
        };

        let resp_s = app.execute_contract(Addr::unchecked("owner"), addr.clone(), &msg, &[]);

        println!("{:?}", resp_s);
    }
}

use cosmwasm_std::{Deps, DepsMut, Addr,to_binary, Env, Empty, entry_point, MessageInfo, StdResult, Response, StdError, SubMsg, BankMsg, WasmMsg, Uint128};
use crate::msg::{ExecuteMsg, OrderListForERC20, OrderListForERC721};
use cw20::{Cw20ExecuteMsg};
use cw721::{Cw721ExecuteMsg};
use crate::state::LIST;

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
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> StdResult<Response>  {
    use ExecuteMsg::*;
    match _msg {
        Register {listforseller} => execute::register(
            deps,
            _info,
            listforseller
        ),
        Exchange {listforbuyer} => execute::exchange(
            deps,
            listforbuyer,
        ),
    }
}  

mod execute {
    use super::*;

    pub fn register(
        deps: DepsMut,
        info: MessageInfo,
        listforseller: OrderListForERC721,
    ) -> StdResult<Response> {
        let curr = LIST.load(deps.storage, listforseller.tokenid);

        match curr {
            Ok(_) => {
                return Err(StdError::generic_err("token is already present"));
            },

            Err(_) => {
                LIST.save(deps.storage, listforseller.tokenid, &listforseller)?;
            }
        }
        Ok(Response::new())
    }

    pub fn exchange(
        deps: DepsMut,
        listforbuyer: OrderListForERC20,
    ) -> StdResult<Response>  {
        
        let mut curr_list = LIST.load(deps.storage, listforbuyer.tokenid_want)?;


        if curr_list.amountof_erc20 != listforbuyer.token_amount {
            return Err(StdError::generic_err("ERC20 token amount not matched"));
        }

        let msg = Cw20ExecuteMsg::TransferFrom {
            owner: listforbuyer.owner.to_string().clone(),
            recipient: curr_list.owner.to_string().clone(),
            amount:  listforbuyer.token_amount.into()
        };

        let exec:Vec<SubMsg> = vec![SubMsg::new(WasmMsg::Execute { 
            contract_addr: listforbuyer.contractaddress.to_string(),
             msg: to_binary(&msg)?,
              funds: vec![] }
        )];

        let mut msg_nft = Cw721ExecuteMsg::TransferNft{
            recipient: listforbuyer.owner.clone().into(),
            token_id: listforbuyer.tokenid_want.to_string(),
        };

        let exec_ERC721: Vec<SubMsg> = vec![SubMsg::new(WasmMsg::Execute {
            contract_addr: curr_list.contractaddress.to_string(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })];

        LIST.remove(deps.storage, listforbuyer.tokenid_want);

        let res = Response::new()
        .add_attribute("action", "transfer")
        .add_attribute("from", listforbuyer.owner.clone())
        .add_attribute("to", curr_list.owner.clone())
        .add_attribute("amount", listforbuyer.token_amount.to_string())
        .add_attribute("action", "transfer_erc721")
        .add_attribute("sender", curr_list.owner)
        .add_attribute("recipient", listforbuyer.owner)
        .add_attribute("token_id", curr_list.tokenid.to_string());

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{ StdError};
    use crate::msg::ExecuteMsg;
    use ExecuteMsg::*;
    use super::*;

    #[test]
    fn register_execute() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate(
            deps.as_ref(),
            env.clone(),
            mock_info("sender", &[]),
            Empty {},
        )
        .unwrap();

        let list = OrderListForERC721 {
            owner: Addr::unchecked("owner"),
            contractaddress:Addr::unchecked("contract"),
            tokenid: 2,
            amountof_erc20:200
        };
        let msg = Register { listforseller: list };

        let res = execute(deps.as_mut(), env.clone(), mock_info("owner", &[]), msg.clone());
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
            contractaddress:Addr::unchecked("contract"),
            tokenid: 2,
            amountof_erc20:200
        };
        let msg = Register { listforseller: list.clone() };

        let res = execute(deps.as_mut(), env.clone(), mock_info("owner", &[]), msg.clone());
        assert_eq!(res, Ok(Response::new()));

        let listForbuyer: OrderListForERC20 = OrderListForERC20 {
            owner: Addr::unchecked("buyer"),
            contractaddress: Addr::unchecked("ERC271contract"),
            token_amount: 200,
            tokenid_want: 2
        };

        let msg = Exchange { listforbuyer: listForbuyer.clone() };
        let res = execute(deps.as_mut(), env.clone(), mock_info("buyer", &[]), msg.clone());
        let responsexpected = Response::new().add_attribute("action", "transfer")
        .add_attribute("from", listForbuyer.owner.clone())
        .add_attribute("to", list.owner.clone())
        .add_attribute("amount", listForbuyer.token_amount.to_string())
        .add_attribute("action", "transfer_erc721")
        .add_attribute("sender", list.owner)
        .add_attribute("recipient", listForbuyer.owner)
        .add_attribute("token_id", list.tokenid.to_string());
        println!("{:?}", res);
        assert_eq!(res.unwrap(), responsexpected);

    }

}
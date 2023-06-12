
#[cfg(test)]
mod multest {
    use cw_multi_test::{App, ContractWrapper, Executor};
    use crate::contract::{execute, instantiate, query};
    use cosmwasm_std::{Addr, Empty, Uint128};
    use crate::msg::{ExecuteMsg::*, OrderListForERC20, OrderListForERC721};
    use cw20_base::contract::{execute as cw20execute , query as cw20query, instantiate as cw20instantiate};
    use cw721_base::entry::{execute as cw721execute, query as cw721query, instantiate as cw721instantiate };
    use cw20_base::msg::{ExecuteMsg as cw20_executeMsg, InstantiateMsg as cw20_instantiateMsg};
    use cw721_base::msg::{ExecuteMsg as cw721_executeMsg, InstantiateMsg as cw721_instantiateMsg};
    use cw20::{Cw20Coin};

     #[test]
    fn multitest() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let code_cw20 = ContractWrapper::new(cw20execute, cw20instantiate, cw20query);
        let code_id_cw20 = app.store_code(Box::new(code_cw20));

        let code_cw721 = ContractWrapper::new(cw721execute, cw721instantiate, cw721query);
        let code_id_cw721 = app.store_code(Box::new(code_cw721));

        let addr_cw20 = app.instantiate_contract(
            code_id_cw20,
            Addr::unchecked("buyer"),
            &cw20_instantiateMsg{
                name: "my".to_string(),
                symbol: "m".to_string(),
                decimals: 3,
                initial_balances: vec![Cw20Coin{
                    address: "buyer".to_string(),
                    amount : Uint128::new(1000)
                }],
                mint:None, 
                marketing: None
            },
            &[],
            "buyer",
            None
        );

        let addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked("owner"),
            &Empty {},
            &[],
            "owner",
            None,
        )
        .unwrap();

        let res_approve_cw20 = app.execute_contract(Addr::unchecked("buyer"), addr_cw20.unwrap(), &cw20_executeMsg::IncreaseAllowance { spender: addr.to_string(), amount: Uint128::new(200), expires: None }, &[]);

        let addr_cw721 = app.instantiate_contract(
            code_id_cw721,
            Addr::unchecked("seller"),
            &cw721_instantiateMsg{
                name:"nft".to_string(),
                symbol:"n".to_string(),
                minter:"seller".to_string()
            },
            &[],
            "seller",
            None
        );

        let re_mint_cw721 = app.execute_contract(Addr::unchecked("seller"), 
        addr_cw721.unwrap().clone(),
        &cw721_executeMsg::Mint { token_id: 2.to_string(), owner: "seller".to_string(), token_uri: None, extension: None },
        &[]
        );

        let re_approve_cw721 = app.execute_contract(Addr::unchecked("seller"), 
        addr_cw721.unwrap().clone(),
         &cw721_executeMsg::Approve { spender: addr.to_string(), token_id: 2.to_string(), expires: None },
        &[],
        );

        let list = OrderListForERC721 {
            owner: Addr::unchecked("seller"),
            contract_address:addr_cw721.unwrap(),
            erc721_token_id: 2,
            amount_of_erc20_want: 200,
        };
        let msg = Register {
            list_for_seller: list,
        };

        let resp = app.execute_contract(Addr::unchecked("owner"), addr.clone(), &msg, &[]);

        let list = OrderListForERC20 {
            owner: Addr::unchecked("buyer"),
            contract_address: addr_cw20.unwrap(),
            amount_of_erc20: 200,
            erc721_token_id_want: 2,
            erc721_contract_address: addr_cw721.unwrap()
        };
        let msg = Exchange {
            list_for_buyer: list,
        };

        let resp_s = app.execute_contract(Addr::unchecked("owner"), addr.clone(), &msg, &[]);

        println!("{:?}", resp_s);
    }
}
















    // use cw20_base::contract::{execute as cw20execute, instantiate as cw20instantiate, query as cw20query};
    // use cw721_metadata_uri::entry::{execute as cw721execute, instantiate as cw721instantiate, query as Cw721query}
    // use cw721_base::{InstantiateMsg as cw721_instantiateMsg, ExecuteMsg as cw721_executeMsg};
    // use cw20_base::msg::{ExecuteMsg as cw20_executeMsg, InstantiateMsg as cw20_instantiateMsg};
    // use cw20::{Cw20Coin, MarketingInfoResponse};
    // use cw721_base::MintMsg; 
    // use cosmwasm_std::Coin;
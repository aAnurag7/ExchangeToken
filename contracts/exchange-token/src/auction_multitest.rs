#[cfg(test)]
mod auction_multitest {
    use crate::contract::{execute, instantiate, query};
    use crate::msg::{ExecuteMsg::*, OrderListForBuyer, OrderListForSeller, QueryMsg, AuctionType::*, TokenType::*};
    use cosmwasm_std::{testing::mock_env, Addr, Empty, Uint128};
    use cw_multi_test::{App, ContractWrapper, Executor};

    use cw20::{BalanceResponse, Cw20Coin};
    use cw20_base::{
        contract::{execute as cw20execute, instantiate as cw20instantiate, query as cw20query},
        msg::{
            ExecuteMsg as cw20_executeMsg, InstantiateMsg as cw20_instantiateMsg,
            QueryMsg as cw20_queryMsg,
        },
    };

    use cw721::{Cw721QueryMsg, OwnerOfResponse};
    use cw721_base::{
        entry::{execute as cw721execute, instantiate as cw721instantiate, query as cw721query},
        msg::InstantiateMsg as cw721_instantiateMsg,
        Extension,
    };
    pub type Cw721ExecuteMsg = cw721_base::ExecuteMsg<Extension, String>;

    #[test]
    fn english_auction() {
        let mut app = App::default();
        let env = mock_env();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let code_cw20 = ContractWrapper::new(cw20execute, cw20instantiate, cw20query);
        let code_id_cw20 = app.store_code(Box::new(code_cw20));

        let code_cw721 = ContractWrapper::new(cw721execute, cw721instantiate, cw721query);
        let code_id_cw721 = app.store_code(Box::new(code_cw721));

        let exchange_owner = Addr::unchecked("owner");
        let seller = Addr::unchecked("seller");
        let buyer = Addr::unchecked("buyer");

        let addr_cw20 = app
            .instantiate_contract(
                code_id_cw20,
                buyer.clone(),
                &cw20_instantiateMsg {
                    name: "mytoken".to_string(),
                    symbol: "myt".to_string(),
                    decimals: 3,
                    initial_balances: vec![Cw20Coin {
                        address: buyer.to_string().clone(),
                        amount: Uint128::new(1000),
                    }],
                    mint: None,
                    marketing: None,
                },
                &[],
                buyer.clone(),
                None,
            )
            .unwrap();

        let addr = app
            .instantiate_contract(
                code_id,
                exchange_owner.clone(),
                &Empty {},
                &[],
                exchange_owner.clone(),
                None,
            )
            .unwrap();

        let _res_approve_cw20 = app.execute_contract(
            buyer.clone(),
            addr_cw20.clone(),
            &cw20_executeMsg::IncreaseAllowance {
                spender: addr.to_string(),
                amount: Uint128::new(200),
                expires: None,
            },
            &[],
        );

        let addr_cw721 = app
            .instantiate_contract(
                code_id_cw721,
                seller.clone(),
                &cw721_instantiateMsg {
                    name: "nftToken".to_string(),
                    symbol: "nyt".to_string(),
                    minter: seller.to_string().clone(),
                },
                &[],
                seller.clone(),
                None,
            )
            .unwrap();

        let _re_mint_cw721 = app.execute_contract(
            seller.clone(),
            addr_cw721.clone(),
            &Cw721ExecuteMsg::Mint {
                token_id: 2.to_string(),
                owner: seller.clone().to_string(),
                token_uri: None,
                extension: None,
            },
            &[],
        );

        let _re_approve_cw721 = app.execute_contract(
            seller.clone(),
            addr_cw721.clone(),
            &Cw721ExecuteMsg::Approve {
                spender: addr.to_string(),
                token_id: 2.to_string(),
                expires: None,
            },
            &[],
        );

        let list = OrderListForSeller {
            owner: seller.clone(),
            contract_address: addr_cw721.clone(),
            erc721_token_id: 2,
            highest_bid: 200,
            end_time: env.block.height + 2,
            start_time: env.block.height,
            highest_bidder: Addr::unchecked(""),
            erc20_amount_after_time: 0,
            auction_type: English,
            sell_token_type: ERC721
        };
        let msg = Register {
            list_for_seller: list.clone(),
        };

        let _resp = app
            .execute_contract(
                Addr::unchecked(exchange_owner.clone()),
                addr.clone(),
                &msg,
                &[],
            )
            .unwrap();

        let list = OrderListForBuyer {
            owner: buyer.clone(),
            contract_address: addr_cw20.clone(),
            amount_of_erc20: 250,
            erc721_token_id_want: 2,
            erc721_contract_address: addr_cw721.clone(),
            buy_token_type: ERC721
        };
        let msg = EnglishAuction {
            list_for_buyer: list.clone(),
        };

        let _resp = app.execute_contract(buyer.clone(), addr.clone(), &msg, &[]);

        let resp: OrderListForSeller = app
            .wrap()
            .query_wasm_smart(
                addr,
                &QueryMsg::OrderList {
                    token_id: 2,
                    contract_address: addr_cw721.clone(),
                },
            )
            .unwrap();

        assert_eq!(
            resp,
            OrderListForSeller {
                owner: seller.clone(),
                contract_address: addr_cw721.clone(),
                erc721_token_id: 2,
                highest_bid: 250,
                end_time: env.block.height + 2,
                start_time: env.block.height,
                highest_bidder: buyer,
                erc20_amount_after_time: 0,
                auction_type: English,
                sell_token_type: ERC721
            }
        );
    }
    
    #[test]
    fn dutch_auction() {
        let mut app = App::default();
        let env = mock_env();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let code_cw20 = ContractWrapper::new(cw20execute, cw20instantiate, cw20query);
        let code_id_cw20 = app.store_code(Box::new(code_cw20));

        let code_cw721 = ContractWrapper::new(cw721execute, cw721instantiate, cw721query);
        let code_id_cw721 = app.store_code(Box::new(code_cw721));

        let exchange_owner = Addr::unchecked("owner");
        let seller = Addr::unchecked("seller");
        let buyer = Addr::unchecked("buyer");

        let addr_cw20 = app
            .instantiate_contract(
                code_id_cw20,
                buyer.clone(),
                &cw20_instantiateMsg {
                    name: "mytoken".to_string(),
                    symbol: "myt".to_string(),
                    decimals: 3,
                    initial_balances: vec![Cw20Coin {
                        address: buyer.to_string().clone(),
                        amount: Uint128::new(1000),
                    }],
                    mint: None,
                    marketing: None,
                },
                &[],
                buyer.clone(),
                None,
            )
            .unwrap();

        let addr = app
            .instantiate_contract(
                code_id,
                exchange_owner.clone(),
                &Empty {},
                &[],
                exchange_owner.clone(),
                None,
            )
            .unwrap();

        let _res_approve_cw20 = app.execute_contract(
            buyer.clone(),
            addr_cw20.clone(),
            &cw20_executeMsg::IncreaseAllowance {
                spender: addr.to_string(),
                amount: Uint128::new(200),
                expires: None,
            },
            &[],
        );

        let addr_cw721 = app
            .instantiate_contract(
                code_id_cw721,
                seller.clone(),
                &cw721_instantiateMsg {
                    name: "nftToken".to_string(),
                    symbol: "nyt".to_string(),
                    minter: seller.to_string().clone(),
                },
                &[],
                seller.clone(),
                None,
            )
            .unwrap();

        let _re_mint_cw721 = app.execute_contract(
            seller.clone(),
            addr_cw721.clone(),
            &Cw721ExecuteMsg::Mint {
                token_id: 2.to_string(),
                owner: seller.clone().to_string(),
                token_uri: None,
                extension: None,
            },
            &[],
        );

        let _re_approve_cw721 = app.execute_contract(
            seller.clone(),
            addr_cw721.clone(),
            &Cw721ExecuteMsg::Approve {
                spender: addr.to_string(),
                token_id: 2.to_string(),
                expires: None,
            },
            &[],
        );

        let list = OrderListForSeller {
            owner: seller.clone(),
            contract_address: addr_cw721.clone(),
            erc721_token_id: 2,
            highest_bid: 200,
            end_time: env.block.height + 2,
            start_time: env.block.height,
            highest_bidder: Addr::unchecked(""),
            erc20_amount_after_time: 60,
            auction_type: Dutch,
            sell_token_type: ERC721
        };
        let msg = Register {
            list_for_seller: list.clone(),
        };

        let _resp = app
            .execute_contract(
                Addr::unchecked(exchange_owner.clone()),
                addr.clone(),
                &msg,
                &[],
            )
            .unwrap();

        let list = OrderListForBuyer {
            owner: buyer.clone(),
            contract_address: addr_cw20.clone(),
            amount_of_erc20: 250,
            erc721_token_id_want: 2,
            erc721_contract_address: addr_cw721.clone(),
            buy_token_type: ERC721
        };
        let msg = DutchExchange { list_for_buyer: list.clone() };

        let _resp = app.execute_contract(buyer.clone(), addr.clone(), &msg, &[]);
        let resp:BalanceResponse = app
        .wrap()
        .query_wasm_smart(addr_cw20, &cw20_queryMsg::Balance { address: seller.to_string() })
        .unwrap();

        let res: OwnerOfResponse= app
        .wrap()
        .query_wasm_smart(
            addr_cw721, 
            &Cw721QueryMsg::OwnerOf { token_id: 2.to_string(), include_expired: None }
        )
        .unwrap();
        assert_eq!(resp, BalanceResponse{balance: Uint128::new(200)});
        assert_eq!(res, OwnerOfResponse { owner: buyer.to_string(), approvals: vec![]})
    }

}

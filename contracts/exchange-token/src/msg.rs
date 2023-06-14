use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Timestamp};

#[cw_serde]
pub enum ExecuteMsg {
    Register {
        list_for_seller: OrderListForERC721
    },
    Exchange {
        list_for_buyer: OrderListForERC20
    },
    EnglishBidRegister {
        list_for_buyer: OrderListForERC20
    },
    ExchangeEnglishBid {
        list_for_buyer: OrderListForERC20
    },
    Clean {
        list_for_seller: OrderListForERC721
    }
}

#[cw_serde]
pub struct OrderListForERC721 {
    pub owner: Addr,
    pub contract_address: Addr,
    pub erc721_token_id: u64,
    pub highest_bid: u64,
    pub time: Timestamp,
    pub highest_bidder: Addr,
    pub erc20_amount_after_time: u64,
    pub dutch_auction: bool,
}

#[cw_serde]
pub struct OrderListForERC20 {
    pub owner:Addr,
    pub contract_address: Addr,
    pub amount_of_erc20: u64,
    pub erc721_token_id_want: u64,
    pub erc721_contract_address: Addr,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(OrderListForERC721)]
    OrderList { token_id: u64, contract_address: Addr}
}
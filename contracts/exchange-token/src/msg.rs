use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub enum ExecuteMsg {
    Register {
        list_for_seller: OrderListForSeller
    },
    Exchange {
        list_for_buyer: OrderListForBuyer
    },
    EnglishAuction {
        list_for_buyer: OrderListForBuyer
    },
    ExchangeEnglishBid {
        list_for_buyer: OrderListForBuyer
    },
    DutchExchange {
        list_for_buyer: OrderListForBuyer
    },
    Clean {
        list_for_seller: OrderListForSeller
    }
}

#[cw_serde]
pub enum AuctionType {
    Fixed, English, Dutch
}

#[cw_serde]
pub enum TokenType {
    ERC20, ERC721
}

#[cw_serde]
pub struct OrderListForSeller {
    pub owner: Addr,
    pub contract_address: Addr,
    pub highest_bidder: Addr,
    pub erc721_token_id: u64,
    pub highest_bid: u64,
    pub end_time: u64,
    pub start_time: u64,
    pub erc20_amount_after_time: u64,
    pub auction_type: AuctionType,
    pub sell_token_type: TokenType
}

#[cw_serde]
pub struct OrderListForBuyer {
    pub owner:Addr,
    pub contract_address: Addr,
    pub amount_of_erc20: u64,
    pub erc721_token_id_want: u64,
    pub erc721_contract_address: Addr,
    pub buy_token_type: TokenType
}

#[cw_serde]
pub struct ExchangeTokenStruct {
    pub erc20_token_sender: Addr,
    pub erc20_token_reciever: Addr,
    pub erc721_token_sender: Addr,
    pub erc721_token_reciever: Addr,
    pub erc20_contract_address: Addr,
    pub erc721_contract_address: Addr,
    pub token_id: u64,
    pub erc20_amount: u64
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(OrderListForSeller)]
    OrderList { token_id: u64, contract_address: Addr}
}
use cosmwasm_schema::{cw_serde};
use cosmwasm_std::Addr;


#[cw_serde]
pub enum ExecuteMsg {
    Register {
        list_for_seller: OrderListForERC721
    },
    Exchange {
        list_for_buyer: OrderListForERC20
    },
}

#[cw_serde]
pub struct OrderListForERC721 {
    pub owner: Addr,
    pub contract_address: Addr,
    pub erc721_token_id: u32,
    pub amount_of_erc20_want: u32,
}

#[cw_serde]
pub struct OrderListForERC20 {
    pub owner:Addr,
    pub contract_address: Addr,
    pub amount_of_erc20: u32,
    pub erc721_token_id_want: u32,
}
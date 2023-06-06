use cosmwasm_schema::{cw_serde};
use cosmwasm_std::Addr;

#[cw_serde]
pub enum ExecuteMsg {
    Exchange { listforseller: OrderListForERC721, listforbuyer: OrderListForERC20},
}

#[cw_serde]
pub struct OrderListForERC721 {
    pub owner:Addr,
    pub tokenid: u32,
    pub amountof_erc20: u32,
}

#[cw_serde]
pub struct OrderListForERC20 {
    pub owner:Addr,
    pub token_amount: u32,
    pub tokenid_want: u32,
}
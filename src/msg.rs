use cosmwasm_schema::{cw_serde};
use cosmwasm_std::Addr;

#[cw_serde]
pub enum ExecuteMsg {
    exchange { listForseller: OrderListForERC721, listForbuyer: OrderListForERC20},
}

#[cw_serde]
pub struct OrderListForERC721 {
    pub tokenid: u32,
    pub contractAddress: Addr,
    pub amountOfERC20: u32,
}

#[cw_serde]
pub struct OrderListForERC20 {
    pub tokenAmount: u32,
    pub contractAddress: Addr,
    pub wantTokenId: u32,
}
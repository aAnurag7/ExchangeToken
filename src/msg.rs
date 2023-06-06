use cosmwasm_schema::{cw_serde};
use cosmwasm_std::Addr;

#[cw_serde]
pub enum ExecuteMsg {
    Exchange {
        signature_seller: Vec<u8>,
        signature_buyer: Vec<u8>,
        listforseller: OrderListForERC721,
        listforbuyer: OrderListForERC20
    },
}

#[cw_serde]
pub struct OrderListForERC721 {
    pub pubkey_seller: Vec<u8>,
    pub owner: Addr,
    pub contractaddress: Addr,
    pub tokenid: u32,
    pub amountof_erc20: u32,
}

#[cw_serde]
pub struct OrderListForERC20 {
    pub pubkey_buyer: Vec<u8>,
    pub owner:Addr,
    pub contractaddress: Addr,
    pub token_amount: u32,
    pub tokenid_want: u32,
}
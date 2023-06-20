use cosmwasm_std::{Addr};
use k256::ecdsa::{Signature, VerifyingKey};

#[derive(Clone, Debug, PartialEq)]
pub enum ExecuteMsg {
    Exchange {
        signature_seller: Signature,
        listforseller: OrderListForERC721,
        signature_buyer: Signature,
        listforbuyer: OrderListForERC20,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrderListForERC721 {
    pub pubkey_seller: VerifyingKey,
    pub message: String,
    pub owner: Addr,
    pub contract_address: Addr,
    pub erc721_token_id: u32,
    pub amount_of_erc20_want: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrderListForERC20 {
    pub pubkey_buyer: VerifyingKey,
    pub message: String,
    pub owner:Addr,
    pub contract_address: Addr,
    pub erc20_token_amount: u32,
    pub erc721_token_id_want: u32,
}

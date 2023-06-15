use cw_storage_plus::{Map};
use crate::msg::OrderListForSeller;
use cosmwasm_std::Addr;

pub const LIST: Map<(u64, Addr), OrderListForSeller> = Map::new("list");


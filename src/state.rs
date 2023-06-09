use cw_storage_plus::Map;
use crate::msg::OrderListForERC721;
use cosmwasm_std::Addr;

pub const LIST: Map<(u32, Addr), OrderListForERC721> = Map::new("list");
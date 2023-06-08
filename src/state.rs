use cw_storage_plus::Map;
use crate::msg::OrderListForERC721;

pub const LIST: Map<u32, OrderListForERC721> = Map::new("list");
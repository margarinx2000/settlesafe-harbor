use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
#[derive(Copy)]
pub enum Status {
    Created,
    Funded,
    Delivered,
    Released,
    Refunded,
    Cancelled,
    Disputed,
}

#[cw_serde]
pub struct Invoice {
    pub seller: Addr,
    pub buyer: Addr,
    pub arbiter: Addr,
    pub amount: Uint128,
    pub pay_deadline: u64,
    pub deliver_deadline: u64,
    pub review_deadline: u64,
    pub review_window: u64,
    pub status: Status,
    pub status_before_dispute: Option<Status>,
    pub description: String,
}

#[cw_serde]
pub struct Config {
    pub escrow_denom: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const INVOICE_COUNT: Item<u64> = Item::new("invoice_count");
pub const INVOICES: Map<u64, Invoice> = Map::new("invoices");

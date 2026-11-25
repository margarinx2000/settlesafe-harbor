use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    /// Denom held in escrow. Defaults to "umec" (native MEC base unit).
    pub escrow_denom: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateInvoice {
        buyer: String,
        arbiter: String,
        amount: Uint128,
        description: String,
        pay_window: u64,
        deliver_window: u64,
        review_window: u64,
    },
    FundInvoice {
        id: u64,
    },
    CancelInvoice {
        id: u64,
    },
    MarkDelivered {
        id: u64,
    },
    Release {
        id: u64,
    },
    ClaimAfterReview {
        id: u64,
    },
    RefundAfterDeadline {
        id: u64,
    },
    RaiseDispute {
        id: u64,
    },
    ResolveDispute {
        id: u64,
        release_to_seller: bool,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(InvoiceResponse)]
    GetInvoice { id: u64 },
    #[returns(CountResponse)]
    InvoiceCount {},
    #[returns(TotalEscrowedResponse)]
    TotalEscrowed {},
}

#[cw_serde]
pub struct InvoiceResponse {
    pub seller: String,
    pub buyer: String,
    pub arbiter: String,
    pub amount: Uint128,
    pub pay_deadline: u64,
    pub deliver_deadline: u64,
    pub review_deadline: u64,
    pub review_window: u64,
    pub status: String,
    pub description: String,
}

#[cw_serde]
pub struct CountResponse {
    pub count: u64,
}

#[cw_serde]
pub struct TotalEscrowedResponse {
    pub amount: Uint128,
    pub denom: String,
}

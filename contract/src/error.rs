use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("SettleSafe: buyer cannot be the seller")]
    BuyerIsSeller,

    #[error("SettleSafe: arbiter must be neutral")]
    ArbiterNotNeutral,

    #[error("SettleSafe: amount is zero")]
    ZeroAmount,

    #[error("SettleSafe: windows must be positive")]
    ZeroWindow,

    #[error("SettleSafe: no such invoice")]
    NoSuchInvoice,

    #[error("SettleSafe: not awaiting funding")]
    NotAwaitingFunding,

    #[error("SettleSafe: only the buyer can fund")]
    OnlyBuyerCanFund,

    #[error("SettleSafe: payment deadline passed")]
    PayDeadlinePassed,

    #[error("SettleSafe: attached funds must equal the invoice amount in the escrow denom")]
    WrongFunds,

    #[error("SettleSafe: not cancellable")]
    NotCancellable,

    #[error("SettleSafe: only a party, or after expiry")]
    NotPartyOrExpired,

    #[error("SettleSafe: not funded")]
    NotFunded,

    #[error("SettleSafe: only the seller can deliver")]
    OnlySellerCanDeliver,

    #[error("SettleSafe: delivery deadline passed")]
    DeliveryDeadlinePassed,

    #[error("SettleSafe: nothing to release")]
    NothingToRelease,

    #[error("SettleSafe: only the buyer can release")]
    OnlyBuyerCanRelease,

    #[error("SettleSafe: not delivered")]
    NotDelivered,

    #[error("SettleSafe: only the seller can claim")]
    OnlySellerCanClaim,

    #[error("SettleSafe: review window still open")]
    ReviewWindowOpen,

    #[error("SettleSafe: delivery window still open")]
    DeliveryWindowOpen,

    #[error("SettleSafe: nothing to dispute")]
    NothingToDispute,

    #[error("SettleSafe: only a party can dispute")]
    OnlyPartyCanDispute,

    #[error("SettleSafe: not disputed")]
    NotDisputed,

    #[error("SettleSafe: only the arbiter can resolve")]
    OnlyArbiterCanResolve,
}

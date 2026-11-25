//! Unit tests mirroring the 27 Forge cases for SettleSafeEscrow.sol
//! (~/workspace/aevum/repo/test/SettleSafeEscrow.t.sol), adapted to the
//! CosmWasm model: escrow funding is the coins attached to FundInvoice, and
//! payouts are asserted on the BankMsg::Send messages in the Response
//! (there is no bank module behind mock dependencies).
//! Plus CosmWasm-specific cases: exact-funds check and zero-window check.

use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    coins, from_json, BankMsg, CosmosMsg, Empty, Env, OwnedDeps, Timestamp, Uint128,
};

use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use crate::msg::{CountResponse, ExecuteMsg, InstantiateMsg, InvoiceResponse, QueryMsg, TotalEscrowedResponse};
use crate::state::Status;

const SELLER: &str = "seller";
const BUYER: &str = "buyer";
const ARBITER: &str = "arbiter";
const STRANGER: &str = "stranger";

const DENOM: &str = "umec";
const AMOUNT: u128 = 250_000_000; // 250 MEC (6 decimals)
const PAY_WINDOW: u64 = 86_400; // 1 day
const DELIVER_WINDOW: u64 = 172_800; // 2 days
const REVIEW_WINDOW: u64 = 259_200; // 3 days
const T0: u64 = 1_700_000_000;

type Deps = OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>;

fn env_at(t: u64) -> Env {
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(t);
    env
}

fn setup() -> Deps {
    let mut deps = mock_dependencies();
    let info = mock_info(SELLER, &[]);
    instantiate(
        deps.as_mut(),
        env_at(T0),
        info,
        InstantiateMsg { escrow_denom: None },
    )
    .unwrap();
    deps
}

fn create_msg() -> ExecuteMsg {
    ExecuteMsg::CreateInvoice {
        buyer: BUYER.to_string(),
        arbiter: ARBITER.to_string(),
        amount: Uint128::new(AMOUNT),
        description: "Logo design package".to_string(),
        pay_window: PAY_WINDOW,
        deliver_window: DELIVER_WINDOW,
        review_window: REVIEW_WINDOW,
    }
}

fn create(deps: &mut Deps) -> u64 {
    let res = execute(deps.as_mut(), env_at(T0), mock_info(SELLER, &[]), create_msg()).unwrap();
    let id_attr = res
        .attributes
        .iter()
        .find(|a| a.key == "id")
        .expect("create returns id attribute");
    id_attr.value.parse().unwrap()
}

fn fund(deps: &mut Deps, id: u64) {
    execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(BUYER, &coins(AMOUNT, DENOM)),
        ExecuteMsg::FundInvoice { id },
    )
    .unwrap();
}

fn create_and_fund(deps: &mut Deps) -> u64 {
    let id = create(deps);
    fund(deps, id);
    id
}

fn deliver(deps: &mut Deps, id: u64) {
    execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(SELLER, &[]),
        ExecuteMsg::MarkDelivered { id },
    )
    .unwrap();
}

fn create_fund_deliver(deps: &mut Deps) -> u64 {
    let id = create_and_fund(deps);
    deliver(deps, id);
    id
}

fn get_invoice(deps: &Deps, id: u64) -> InvoiceResponse {
    let bin = query(deps.as_ref(), env_at(T0), QueryMsg::GetInvoice { id }).unwrap();
    from_json(bin).unwrap()
}

fn status(deps: &Deps, id: u64) -> String {
    get_invoice(deps, id).status
}

/// Assert the response carries exactly one BankMsg::Send of AMOUNT to `to`.
fn assert_payout(res: &cosmwasm_std::Response, to: &str) {
    assert_eq!(res.messages.len(), 1, "exactly one payout message");
    let expected = CosmosMsg::Bank(BankMsg::Send {
        to_address: to.to_string(),
        amount: coins(AMOUNT, DENOM),
    });
    assert_eq!(res.messages[0].msg, expected, "payout goes to the right party in full");
}

// ---------------------------------------------------------------
// happy paths (Forge: testCreateStoresTerms et al.)
// ---------------------------------------------------------------

#[test]
fn create_stores_terms() {
    let mut deps = setup();
    let id = create(&mut deps);
    let inv = get_invoice(&deps, id);
    assert_eq!(inv.seller, SELLER);
    assert_eq!(inv.buyer, BUYER);
    assert_eq!(inv.arbiter, ARBITER);
    assert_eq!(inv.amount, Uint128::new(AMOUNT));
    assert_eq!(inv.status, format!("{:?}", Status::Created));
    assert_eq!(inv.pay_deadline, T0 + PAY_WINDOW);
    assert_eq!(inv.deliver_deadline, T0 + PAY_WINDOW + DELIVER_WINDOW);
    assert_eq!(inv.description, "Logo design package");

    let bin = query(deps.as_ref(), env_at(T0), QueryMsg::InvoiceCount {}).unwrap();
    let count: CountResponse = from_json(bin).unwrap();
    assert_eq!(count.count, 1);
}

#[test]
fn fund_locks_exact_amount() {
    let mut deps = setup();
    let id = create_and_fund(&mut deps);
    assert_eq!(status(&deps, id), format!("{:?}", Status::Funded));

    // TotalEscrowed reads the contract's bank balance of the escrow denom.
    deps.querier
        .update_balance(mock_env().contract.address, coins(AMOUNT, DENOM));
    let bin = query(deps.as_ref(), env_at(T0), QueryMsg::TotalEscrowed {}).unwrap();
    let total: TotalEscrowedResponse = from_json(bin).unwrap();
    assert_eq!(total.amount, Uint128::new(AMOUNT));
    assert_eq!(total.denom, DENOM);
}

#[test]
fn happy_path_release_after_delivery() {
    let mut deps = setup();
    let id = create_fund_deliver(&mut deps);
    let res = execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(BUYER, &[]),
        ExecuteMsg::Release { id },
    )
    .unwrap();
    assert_payout(&res, SELLER);
    assert_eq!(status(&deps, id), format!("{:?}", Status::Released));
}

#[test]
fn buyer_can_release_early_from_funded() {
    let mut deps = setup();
    let id = create_and_fund(&mut deps);
    let res = execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(BUYER, &[]),
        ExecuteMsg::Release { id },
    )
    .unwrap();
    assert_payout(&res, SELLER);
    assert_eq!(status(&deps, id), format!("{:?}", Status::Released));
}

#[test]
fn seller_claims_after_review_window() {
    let mut deps = setup();
    let id = create_fund_deliver(&mut deps);
    let inv = get_invoice(&deps, id);
    assert_eq!(inv.review_deadline, T0 + REVIEW_WINDOW);
    let res = execute(
        deps.as_mut(),
        env_at(inv.review_deadline + 1),
        mock_info(SELLER, &[]),
        ExecuteMsg::ClaimAfterReview { id },
    )
    .unwrap();
    assert_payout(&res, SELLER);
    assert_eq!(status(&deps, id), format!("{:?}", Status::Released));
}

// ---------------------------------------------------------------
// refund / timeout paths
// ---------------------------------------------------------------

#[test]
fn refund_when_seller_never_delivers() {
    let mut deps = setup();
    let id = create_and_fund(&mut deps);
    let inv = get_invoice(&deps, id);
    // anyone may trigger it; the money can only go back to the buyer
    let res = execute(
        deps.as_mut(),
        env_at(inv.deliver_deadline + 1),
        mock_info(STRANGER, &[]),
        ExecuteMsg::RefundAfterDeadline { id },
    )
    .unwrap();
    assert_payout(&res, BUYER);
    assert_eq!(status(&deps, id), format!("{:?}", Status::Refunded));
}

#[test]
fn cancel_before_funding_by_either_party() {
    let mut deps = setup();
    let id = create(&mut deps);
    execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(BUYER, &[]),
        ExecuteMsg::CancelInvoice { id },
    )
    .unwrap();
    assert_eq!(status(&deps, id), format!("{:?}", Status::Cancelled));
}

#[test]
fn expired_unfunded_invoice_can_be_closed_by_anyone() {
    let mut deps = setup();
    let id = create(&mut deps);
    let inv = get_invoice(&deps, id);
    execute(
        deps.as_mut(),
        env_at(inv.pay_deadline + 1),
        mock_info(STRANGER, &[]),
        ExecuteMsg::CancelInvoice { id },
    )
    .unwrap();
    assert_eq!(status(&deps, id), format!("{:?}", Status::Cancelled));
}

#[test]
fn stranger_cannot_cancel_before_expiry() {
    let mut deps = setup();
    let id = create(&mut deps);
    let err = execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(STRANGER, &[]),
        ExecuteMsg::CancelInvoice { id },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::NotPartyOrExpired);
}

// ---------------------------------------------------------------
// disputes
// ---------------------------------------------------------------

#[test]
fn dispute_resolved_to_seller() {
    let mut deps = setup();
    let id = create_fund_deliver(&mut deps);
    execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(BUYER, &[]),
        ExecuteMsg::RaiseDispute { id },
    )
    .unwrap();
    assert_eq!(status(&deps, id), format!("{:?}", Status::Disputed));
    let res = execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(ARBITER, &[]),
        ExecuteMsg::ResolveDispute {
            id,
            release_to_seller: true,
        },
    )
    .unwrap();
    assert_payout(&res, SELLER);
    assert_eq!(status(&deps, id), format!("{:?}", Status::Released));
}

#[test]
fn dispute_resolved_to_buyer() {
    let mut deps = setup();
    let id = create_and_fund(&mut deps);
    execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(SELLER, &[]),
        ExecuteMsg::RaiseDispute { id },
    )
    .unwrap();
    let res = execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(ARBITER, &[]),
        ExecuteMsg::ResolveDispute {
            id,
            release_to_seller: false,
        },
    )
    .unwrap();
    assert_payout(&res, BUYER);
    assert_eq!(status(&deps, id), format!("{:?}", Status::Refunded));
}

// ---------------------------------------------------------------
// unauthorized access / invalid transitions
// ---------------------------------------------------------------

#[test]
fn stranger_cannot_fund() {
    let mut deps = setup();
    let id = create(&mut deps);
    let err = execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(STRANGER, &coins(AMOUNT, DENOM)),
        ExecuteMsg::FundInvoice { id },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::OnlyBuyerCanFund);
}

#[test]
fn seller_cannot_fund_own_invoice() {
    let mut deps = setup();
    let id = create(&mut deps);
    let err = execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(SELLER, &coins(AMOUNT, DENOM)),
        ExecuteMsg::FundInvoice { id },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::OnlyBuyerCanFund);
}

#[test]
fn fund_after_pay_deadline_reverts() {
    let mut deps = setup();
    let id = create(&mut deps);
    let inv = get_invoice(&deps, id);
    let err = execute(
        deps.as_mut(),
        env_at(inv.pay_deadline + 1),
        mock_info(BUYER, &coins(AMOUNT, DENOM)),
        ExecuteMsg::FundInvoice { id },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::PayDeadlinePassed);
}

#[test]
fn fund_twice_reverts() {
    let mut deps = setup();
    let id = create_and_fund(&mut deps);
    let err = execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(BUYER, &coins(AMOUNT, DENOM)),
        ExecuteMsg::FundInvoice { id },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::NotAwaitingFunding);
}

#[test]
fn fund_with_wrong_amount_reverts() {
    let mut deps = setup();
    let id = create(&mut deps);
    let err = execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(BUYER, &coins(AMOUNT - 1, DENOM)),
        ExecuteMsg::FundInvoice { id },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::WrongFunds);
}

#[test]
fn fund_with_wrong_denom_reverts() {
    let mut deps = setup();
    let id = create(&mut deps);
    let err = execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(BUYER, &coins(AMOUNT, "uother")),
        ExecuteMsg::FundInvoice { id },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::WrongFunds);
}

#[test]
fn stranger_cannot_release() {
    let mut deps = setup();
    let id = create_fund_deliver(&mut deps);
    let err = execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(STRANGER, &[]),
        ExecuteMsg::Release { id },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::OnlyBuyerCanRelease);
}

#[test]
fn seller_cannot_release() {
    let mut deps = setup();
    let id = create_fund_deliver(&mut deps);
    let err = execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(SELLER, &[]),
        ExecuteMsg::Release { id },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::OnlyBuyerCanRelease);
}

#[test]
fn buyer_cannot_mark_delivered() {
    let mut deps = setup();
    let id = create_and_fund(&mut deps);
    let err = execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(BUYER, &[]),
        ExecuteMsg::MarkDelivered { id },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::OnlySellerCanDeliver);
}

#[test]
fn claim_before_review_ends_reverts() {
    let mut deps = setup();
    let id = create_fund_deliver(&mut deps);
    let err = execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(SELLER, &[]),
        ExecuteMsg::ClaimAfterReview { id },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::ReviewWindowOpen);
}

#[test]
fn refund_before_delivery_deadline_reverts() {
    let mut deps = setup();
    let id = create_and_fund(&mut deps);
    let err = execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(BUYER, &[]),
        ExecuteMsg::RefundAfterDeadline { id },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::DeliveryWindowOpen);
}

#[test]
fn deliver_after_deadline_reverts() {
    let mut deps = setup();
    let id = create_and_fund(&mut deps);
    let inv = get_invoice(&deps, id);
    let err = execute(
        deps.as_mut(),
        env_at(inv.deliver_deadline + 1),
        mock_info(SELLER, &[]),
        ExecuteMsg::MarkDelivered { id },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::DeliveryDeadlinePassed);
}

#[test]
fn non_arbiter_cannot_resolve_dispute() {
    let mut deps = setup();
    let id = create_fund_deliver(&mut deps);
    execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(BUYER, &[]),
        ExecuteMsg::RaiseDispute { id },
    )
    .unwrap();
    let err = execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(BUYER, &[]),
        ExecuteMsg::ResolveDispute {
            id,
            release_to_seller: true,
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::OnlyArbiterCanResolve);
}

#[test]
fn stranger_cannot_raise_dispute() {
    let mut deps = setup();
    let id = create_and_fund(&mut deps);
    let err = execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(STRANGER, &[]),
        ExecuteMsg::RaiseDispute { id },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::OnlyPartyCanDispute);
}

#[test]
fn dispute_freezes_release() {
    let mut deps = setup();
    let id = create_fund_deliver(&mut deps);
    execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(SELLER, &[]),
        ExecuteMsg::RaiseDispute { id },
    )
    .unwrap();
    let err = execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(BUYER, &[]),
        ExecuteMsg::Release { id },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::NothingToRelease);
}

#[test]
fn cancel_funded_invoice_reverts() {
    let mut deps = setup();
    let id = create_and_fund(&mut deps);
    let err = execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(SELLER, &[]),
        ExecuteMsg::CancelInvoice { id },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::NotCancellable);
}

#[test]
fn create_rejects_self_as_buyer() {
    let mut deps = setup();
    let mut msg = create_msg();
    if let ExecuteMsg::CreateInvoice { buyer, .. } = &mut msg {
        *buyer = SELLER.to_string();
    }
    let err = execute(deps.as_mut(), env_at(T0), mock_info(SELLER, &[]), msg).unwrap_err();
    assert_eq!(err, ContractError::BuyerIsSeller);
}

#[test]
fn create_rejects_non_neutral_arbiter() {
    let mut deps = setup();
    let mut msg = create_msg();
    if let ExecuteMsg::CreateInvoice { arbiter, .. } = &mut msg {
        *arbiter = BUYER.to_string();
    }
    let err = execute(deps.as_mut(), env_at(T0), mock_info(SELLER, &[]), msg).unwrap_err();
    assert_eq!(err, ContractError::ArbiterNotNeutral);
}

#[test]
fn create_rejects_zero_amount() {
    let mut deps = setup();
    let mut msg = create_msg();
    if let ExecuteMsg::CreateInvoice { amount, .. } = &mut msg {
        *amount = Uint128::zero();
    }
    let err = execute(deps.as_mut(), env_at(T0), mock_info(SELLER, &[]), msg).unwrap_err();
    assert_eq!(err, ContractError::ZeroAmount);
}

#[test]
fn create_rejects_zero_window() {
    let mut deps = setup();
    let mut msg = create_msg();
    if let ExecuteMsg::CreateInvoice { pay_window, .. } = &mut msg {
        *pay_window = 0;
    }
    let err = execute(deps.as_mut(), env_at(T0), mock_info(SELLER, &[]), msg).unwrap_err();
    assert_eq!(err, ContractError::ZeroWindow);
}

#[test]
fn no_such_invoice_errors() {
    let mut deps = setup();
    let err = execute(
        deps.as_mut(),
        env_at(T0),
        mock_info(BUYER, &coins(AMOUNT, DENOM)),
        ExecuteMsg::FundInvoice { id: 99 },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::NoSuchInvoice);
    assert!(query(deps.as_ref(), env_at(T0), QueryMsg::GetInvoice { id: 99 }).is_err());
}

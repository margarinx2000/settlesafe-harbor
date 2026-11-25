use cosmwasm_std::{
    entry_point, to_json_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128,
};

use crate::error::ContractError;
use crate::msg::{
    CountResponse, ExecuteMsg, InstantiateMsg, InvoiceResponse, QueryMsg,
    TotalEscrowedResponse,
};
use crate::state::{Config, Invoice, Status, CONFIG, INVOICES, INVOICE_COUNT};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let denom = msg.escrow_denom.unwrap_or_else(|| "umec".to_string());
    CONFIG.save(deps.storage, &Config { escrow_denom: denom })?;
    INVOICE_COUNT.save(deps.storage, &0u64)?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateInvoice {
            buyer,
            arbiter,
            amount,
            description,
            pay_window,
            deliver_window,
            review_window,
        } => create_invoice(
            deps,
            env,
            info,
            buyer,
            arbiter,
            amount,
            description,
            pay_window,
            deliver_window,
            review_window,
        ),
        ExecuteMsg::FundInvoice { id } => fund_invoice(deps, env, info, id),
        ExecuteMsg::CancelInvoice { id } => cancel_invoice(deps, env, info, id),
        ExecuteMsg::MarkDelivered { id } => mark_delivered(deps, env, info, id),
        ExecuteMsg::Release { id } => release(deps, info, id),
        ExecuteMsg::ClaimAfterReview { id } => claim_after_review(deps, env, info, id),
        ExecuteMsg::RefundAfterDeadline { id } => refund_after_deadline(deps, env, id),
        ExecuteMsg::RaiseDispute { id } => raise_dispute(deps, info, id),
        ExecuteMsg::ResolveDispute {
            id,
            release_to_seller,
        } => resolve_dispute(deps, info, id, release_to_seller),
    }
}

fn load_invoice(deps: Deps, id: u64) -> Result<Invoice, ContractError> {
    INVOICES
        .may_load(deps.storage, id)?
        .ok_or(ContractError::NoSuchInvoice)
}

fn pay(to: &Addr, amount: Uint128, denom: &str) -> BankMsg {
    BankMsg::Send {
        to_address: to.to_string(),
        amount: vec![Coin {
            denom: denom.to_string(),
            amount,
        }],
    }
}

#[allow(clippy::too_many_arguments)]
fn create_invoice(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    buyer: String,
    arbiter: String,
    amount: Uint128,
    description: String,
    pay_window: u64,
    deliver_window: u64,
    review_window: u64,
) -> Result<Response, ContractError> {
    let buyer = deps.api.addr_validate(&buyer)?;
    let arbiter = deps.api.addr_validate(&arbiter)?;
    if buyer == info.sender {
        return Err(ContractError::BuyerIsSeller);
    }
    if arbiter == info.sender || arbiter == buyer {
        return Err(ContractError::ArbiterNotNeutral);
    }
    if amount.is_zero() {
        return Err(ContractError::ZeroAmount);
    }
    if pay_window == 0 || deliver_window == 0 || review_window == 0 {
        return Err(ContractError::ZeroWindow);
    }

    let now = env.block.time.seconds();
    let id = INVOICE_COUNT.load(deps.storage)?;
    let inv = Invoice {
        seller: info.sender.clone(),
        buyer: buyer.clone(),
        arbiter,
        amount,
        pay_deadline: now + pay_window,
        deliver_deadline: now + pay_window + deliver_window,
        review_deadline: 0,
        review_window,
        status: Status::Created,
        status_before_dispute: None,
        description,
    };
    INVOICES.save(deps.storage, id, &inv)?;
    INVOICE_COUNT.save(deps.storage, &(id + 1))?;

    Ok(Response::new()
        .add_attribute("action", "create_invoice")
        .add_attribute("id", id.to_string())
        .add_attribute("seller", info.sender.to_string())
        .add_attribute("buyer", buyer.to_string())
        .add_attribute("amount", amount.to_string()))
}

fn fund_invoice(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let mut inv = load_invoice(deps.as_ref(), id)?;
    if inv.status != Status::Created {
        return Err(ContractError::NotAwaitingFunding);
    }
    if info.sender != inv.buyer {
        return Err(ContractError::OnlyBuyerCanFund);
    }
    if env.block.time.seconds() > inv.pay_deadline {
        return Err(ContractError::PayDeadlinePassed);
    }
    // The attached funds ARE the escrow deposit: exactly `amount` of the denom.
    let paid = info
        .funds
        .iter()
        .filter(|c| c.denom == cfg.escrow_denom)
        .map(|c| c.amount)
        .sum::<Uint128>();
    if info.funds.len() != 1 || paid != inv.amount {
        return Err(ContractError::WrongFunds);
    }

    inv.status = Status::Funded;
    INVOICES.save(deps.storage, id, &inv)?;

    Ok(Response::new()
        .add_attribute("action", "fund_invoice")
        .add_attribute("id", id.to_string())
        .add_attribute("buyer", info.sender.to_string())
        .add_attribute("amount", inv.amount.to_string()))
}

fn cancel_invoice(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let mut inv = load_invoice(deps.as_ref(), id)?;
    if inv.status != Status::Created {
        return Err(ContractError::NotCancellable);
    }
    let is_party = info.sender == inv.seller || info.sender == inv.buyer;
    if !is_party && env.block.time.seconds() <= inv.pay_deadline {
        return Err(ContractError::NotPartyOrExpired);
    }

    inv.status = Status::Cancelled;
    INVOICES.save(deps.storage, id, &inv)?;

    Ok(Response::new()
        .add_attribute("action", "cancel_invoice")
        .add_attribute("id", id.to_string())
        .add_attribute("by", info.sender.to_string()))
}

fn mark_delivered(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let mut inv = load_invoice(deps.as_ref(), id)?;
    if inv.status != Status::Funded {
        return Err(ContractError::NotFunded);
    }
    if info.sender != inv.seller {
        return Err(ContractError::OnlySellerCanDeliver);
    }
    let now = env.block.time.seconds();
    if now > inv.deliver_deadline {
        return Err(ContractError::DeliveryDeadlinePassed);
    }

    inv.status = Status::Delivered;
    inv.review_deadline = now + inv.review_window;
    let review_deadline = inv.review_deadline;
    INVOICES.save(deps.storage, id, &inv)?;

    Ok(Response::new()
        .add_attribute("action", "mark_delivered")
        .add_attribute("id", id.to_string())
        .add_attribute("review_deadline", review_deadline.to_string()))
}

fn release(deps: DepsMut, info: MessageInfo, id: u64) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let mut inv = load_invoice(deps.as_ref(), id)?;
    if inv.status != Status::Funded && inv.status != Status::Delivered {
        return Err(ContractError::NothingToRelease);
    }
    if info.sender != inv.buyer {
        return Err(ContractError::OnlyBuyerCanRelease);
    }

    inv.status = Status::Released;
    let (seller, amount) = (inv.seller.clone(), inv.amount);
    INVOICES.save(deps.storage, id, &inv)?;

    Ok(Response::new()
        .add_message(pay(&seller, amount, &cfg.escrow_denom))
        .add_attribute("action", "release")
        .add_attribute("id", id.to_string())
        .add_attribute("seller", seller.to_string())
        .add_attribute("amount", amount.to_string()))
}

fn claim_after_review(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let mut inv = load_invoice(deps.as_ref(), id)?;
    if inv.status != Status::Delivered {
        return Err(ContractError::NotDelivered);
    }
    if info.sender != inv.seller {
        return Err(ContractError::OnlySellerCanClaim);
    }
    if env.block.time.seconds() <= inv.review_deadline {
        return Err(ContractError::ReviewWindowOpen);
    }

    inv.status = Status::Released;
    let (seller, amount) = (inv.seller.clone(), inv.amount);
    INVOICES.save(deps.storage, id, &inv)?;

    Ok(Response::new()
        .add_message(pay(&seller, amount, &cfg.escrow_denom))
        .add_attribute("action", "claim_after_review")
        .add_attribute("id", id.to_string())
        .add_attribute("seller", seller.to_string())
        .add_attribute("amount", amount.to_string()))
}

fn refund_after_deadline(
    deps: DepsMut,
    env: Env,
    id: u64,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let mut inv = load_invoice(deps.as_ref(), id)?;
    if inv.status != Status::Funded {
        return Err(ContractError::NotFunded);
    }
    if env.block.time.seconds() <= inv.deliver_deadline {
        return Err(ContractError::DeliveryWindowOpen);
    }

    inv.status = Status::Refunded;
    let (buyer, amount) = (inv.buyer.clone(), inv.amount);
    INVOICES.save(deps.storage, id, &inv)?;

    Ok(Response::new()
        .add_message(pay(&buyer, amount, &cfg.escrow_denom))
        .add_attribute("action", "refund_after_deadline")
        .add_attribute("id", id.to_string())
        .add_attribute("buyer", buyer.to_string())
        .add_attribute("amount", amount.to_string()))
}

fn raise_dispute(deps: DepsMut, info: MessageInfo, id: u64) -> Result<Response, ContractError> {
    let mut inv = load_invoice(deps.as_ref(), id)?;
    if inv.status != Status::Funded && inv.status != Status::Delivered {
        return Err(ContractError::NothingToDispute);
    }
    if info.sender != inv.buyer && info.sender != inv.seller {
        return Err(ContractError::OnlyPartyCanDispute);
    }

    inv.status_before_dispute = Some(inv.status);
    inv.status = Status::Disputed;
    INVOICES.save(deps.storage, id, &inv)?;

    Ok(Response::new()
        .add_attribute("action", "raise_dispute")
        .add_attribute("id", id.to_string())
        .add_attribute("by", info.sender.to_string()))
}

fn resolve_dispute(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
    release_to_seller: bool,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let mut inv = load_invoice(deps.as_ref(), id)?;
    if inv.status != Status::Disputed {
        return Err(ContractError::NotDisputed);
    }
    if info.sender != inv.arbiter {
        return Err(ContractError::OnlyArbiterCanResolve);
    }

    let (to, amount) = if release_to_seller {
        inv.status = Status::Released;
        (inv.seller.clone(), inv.amount)
    } else {
        inv.status = Status::Refunded;
        (inv.buyer.clone(), inv.amount)
    };
    INVOICES.save(deps.storage, id, &inv)?;

    Ok(Response::new()
        .add_message(pay(&to, amount, &cfg.escrow_denom))
        .add_attribute("action", "resolve_dispute")
        .add_attribute("id", id.to_string())
        .add_attribute("released_to_seller", release_to_seller.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetInvoice { id } => {
            let inv = INVOICES.may_load(deps.storage, id)?.ok_or_else(|| {
                cosmwasm_std::StdError::not_found("invoice")
            })?;
            to_json_binary(&InvoiceResponse {
                seller: inv.seller.to_string(),
                buyer: inv.buyer.to_string(),
                arbiter: inv.arbiter.to_string(),
                amount: inv.amount,
                pay_deadline: inv.pay_deadline,
                deliver_deadline: inv.deliver_deadline,
                review_deadline: inv.review_deadline,
                review_window: inv.review_window,
                status: format!("{:?}", inv.status),
                description: inv.description,
            })
        }
        QueryMsg::InvoiceCount {} => {
            let count = INVOICE_COUNT.load(deps.storage)?;
            to_json_binary(&CountResponse { count })
        }
        QueryMsg::TotalEscrowed {} => {
            let cfg = CONFIG.load(deps.storage)?;
            let bal = deps
                .querier
                .query_balance(env.contract.address, cfg.escrow_denom.clone())?;
            to_json_binary(&TotalEscrowedResponse {
                amount: bal.amount,
                denom: cfg.escrow_denom,
            })
        }
    }
}

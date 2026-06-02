#![cfg(test)]

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, String, vec};

use crate::{BcForgeToken, BcForgeTokenClient, Recipient, TokenError};

fn setup(env: &Env) -> (BcForgeTokenClient<'_>, Address) {
    let contract_id = env.register(BcForgeToken, ());
    let client = BcForgeTokenClient::new(env, &contract_id);
    let admin = Address::generate(env);

    client.initialize(
        &admin,
        &7,
        &String::from_str(env, "bc-forge Token"),
        &String::from_str(env, "SFG"),
    );

    (client, admin)
}

#[test]
fn test_mint_transfer_and_supply() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);
    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.mint(&from, &1_000);
    client.transfer(&from, &to, &300);

    assert_eq!(client.balance(&from), 700);
    assert_eq!(client.balance(&to), 300);
    assert_eq!(client.supply(), 1_000);
}

#[test]
fn test_approve_and_transfer_from() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let receiver = Address::generate(&env);

    client.mint(&owner, &1_000);
    client.approve(&owner, &spender, &500, &0);
    client.transfer_from(&spender, &owner, &receiver, &200);

    assert_eq!(client.balance(&owner), 800);
    assert_eq!(client.balance(&receiver), 200);
    assert_eq!(client.allowance(&owner, &spender), 300);
}

#[test]
fn test_transfer_ownership_updates_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);
    let new_admin = Address::generate(&env);

    client.transfer_ownership(&new_admin);

    assert_eq!(client.admin(), new_admin);
}

#[test]
fn test_batch_mint_single_recipient() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);
    let recipient = Address::generate(&env);

    let recipients = vec![
        &env,
        Recipient {
            address: recipient.clone(),
            amount: 1000,
        },
    ];

    client.batch_mint(&recipients);

    assert_eq!(client.balance(&recipient), 1000);
    assert_eq!(client.supply(), 1000);
}

#[test]
fn test_batch_mint_multiple_recipients() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);
    let r1 = Address::generate(&env);
    let r2 = Address::generate(&env);
    let r3 = Address::generate(&env);

    let recipients = vec![
        &env,
        Recipient {
            address: r1.clone(),
            amount: 100,
        },
        Recipient {
            address: r2.clone(),
            amount: 200,
        },
        Recipient {
            address: r3.clone(),
            amount: 300,
        },
    ];

    client.batch_mint(&recipients);

    assert_eq!(client.balance(&r1), 100);
    assert_eq!(client.balance(&r2), 200);
    assert_eq!(client.balance(&r3), 300);
    assert_eq!(client.supply(), 600);
}

#[test]
fn test_batch_mint_rejects_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);
    let r1 = Address::generate(&env);
    let r2 = Address::generate(&env);

    let recipients = vec![
        &env,
        Recipient {
            address: r1.clone(),
            amount: 100,
        },
        Recipient {
            address: r2.clone(),
            amount: 0, // Zero amount
        },
    ];

    let res = client.try_batch_mint(&recipients);
    assert_eq!(
        res,
        Err(Ok(TokenError::InvalidAmount))
    );

    // Verify atomic rollback (no tokens minted)
    assert_eq!(client.balance(&r1), 0);
    assert_eq!(client.balance(&r2), 0);
    assert_eq!(client.supply(), 0);
}

#[test]
fn test_batch_mint_while_paused_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);
    let recipient = Address::generate(&env);

    let recipients = vec![
        &env,
        Recipient {
            address: recipient.clone(),
            amount: 100,
        },
    ];

    client.pause();

    let res = client.try_batch_mint(&recipients);
    assert_eq!(
        res,
        Err(Ok(TokenError::ContractPaused))
    );

    // Verify no tokens minted
    assert_eq!(client.balance(&recipient), 0);
    assert_eq!(client.supply(), 0);
}

#![cfg(test)]

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{vec, Address, Env, String, Vec};

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
fn test_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);
    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.mint(&from, &1000);
    client.transfer(&from, &to, &300);

    assert_eq!(client.balance(&from), 700);
    assert_eq!(client.balance(&to), 300);
    assert_eq!(client.supply(), 1000);
}

#[test]
fn test_batch_transfer_multiple_recipients() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);
    let from = Address::generate(&env);
    let recipient_a = Address::generate(&env);
    let recipient_b = Address::generate(&env);
    let recipient_c = Address::generate(&env);

    client.mint(&from, &1000);

    let recipients = vec![
        &env,
        (recipient_a.clone(), 100_i128),
        (recipient_b.clone(), 250_i128),
        (recipient_c.clone(), 50_i128),
    ];
    client.batch_transfer(&from, &recipients);

    assert_eq!(client.balance(&from), 600);
    assert_eq!(client.balance(&recipient_a), 100);
    assert_eq!(client.balance(&recipient_b), 250);
    assert_eq!(client.balance(&recipient_c), 50);
    assert_eq!(client.supply(), 1000);
}

#[test]
fn test_batch_transfer_rejects_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);
    let from = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.mint(&from, &1000);

    let recipients = vec![&env, (recipient.clone(), 0_i128)];
    assert_eq!(
        client.try_batch_transfer(&from, &recipients),
        Err(Ok(soroban_sdk::Error::from_contract_error(
            TokenError::InvalidAmount as u32
        )))
    );
    assert_eq!(client.balance(&from), 1000);
    assert_eq!(client.balance(&recipient), 0);
}

#[test]
fn test_batch_transfer_rejects_insufficient_balance_before_moving_tokens() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);
    let from = Address::generate(&env);
    let recipient_a = Address::generate(&env);
    let recipient_b = Address::generate(&env);

    client.mint(&from, &100);

    let recipients = vec![
        &env,
        (recipient_a.clone(), 80_i128),
        (recipient_b.clone(), 40_i128),
    ];
    assert_eq!(
        client.try_batch_transfer(&from, &recipients),
        Err(Ok(soroban_sdk::Error::from_contract_error(
            TokenError::InsufficientBalance as u32
        )))
    );
    assert_eq!(client.balance(&from), 100);
    assert_eq!(client.balance(&recipient_a), 0);
    assert_eq!(client.balance(&recipient_b), 0);
}

#[test]
fn test_batch_transfer_while_paused_returns_error() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);
    let from = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.mint(&from, &100);
    client.pause();

    let recipients: Vec<(Address, i128)> = vec![&env, (recipient, 10_i128)];
    assert_eq!(
        client.try_batch_transfer(&from, &recipients),
        Err(Ok(soroban_sdk::Error::from_contract_error(
            TokenError::ContractPaused as u32
        )))
    );
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
        Err(Ok(soroban_sdk::Error::from_contract_error(
            TokenError::InvalidAmount as u32
        )))
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
        Err(Ok(soroban_sdk::Error::from_contract_error(
            TokenError::ContractPaused as u32
        )))
    );

    // Verify no tokens minted
    assert_eq!(client.balance(&recipient), 0);
    assert_eq!(client.supply(), 0);
}

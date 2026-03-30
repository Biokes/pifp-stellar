#![cfg(test)]

use crate::test_utils::{create_token, setup_test};
use crate::{Error, ProjectStatus, Role};
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Bytes, Vec};

fn dummy_metadata(env: &soroban_sdk::Env) -> Bytes {
    Bytes::from_slice(env, b"bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi")
}

#[test]
fn test_whitelist_funding_restricted() {
    let (env, client, admin) = setup_test();
    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, &admin);
    let accepted_tokens = Vec::from_array(&env, [token.address.clone()]);

    client.grant_role(&admin, &creator, &Role::ProjectManager);

    // Register a private project
    let project = client.register_project(
        &creator,
        &accepted_tokens,
        &1000,
        &[0u8; 32].into(),
        &dummy_metadata(&env),
        &(env.ledger().timestamp() + 10000),
        &true, // is_private
        &0u32,
    );

    // Attempt deposit from non-whitelisted donor
    token.mint(&donor, &500);
    let result = client.try_deposit(&project.id, &donor, &token.address, &500);

    assert!(result.is_err());
    // Error::NotWhitelisted = 26
}

#[test]
fn test_whitelist_funding_allowed() {
    let (env, client, admin) = setup_test();
    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, &admin);
    let accepted_tokens = Vec::from_array(&env, [token.address.clone()]);

    client.grant_role(&admin, &creator, &Role::ProjectManager);

    let project = client.register_project(
        &creator,
        &accepted_tokens,
        &1000,
        &[0u8; 32].into(),
        &dummy_metadata(&env),
        &(env.ledger().timestamp() + 10000),
        &true,
        &0u32,
    );

    // Add donor to whitelist
    client.add_to_whitelist(&creator, &project.id, &donor);

    // Deposit should now work
    token.mint(&donor, &500);
    client.deposit(&project.id, &donor, &token.address, &500);

    let balance = client.get_balance(&project.id, &token.address);
    assert_eq!(balance, 500);
}

#[test]
fn test_whitelist_management_auth() {
    let (env, client, admin) = setup_test();
    let creator = Address::generate(&env);
    let stranger = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, &admin);
    let accepted_tokens = Vec::from_array(&env, [token.address.clone()]);

    client.grant_role(&admin, &creator, &Role::ProjectManager);

    let project = client.register_project(
        &creator,
        &accepted_tokens,
        &1000,
        &[0u8; 32].into(),
        &dummy_metadata(&env),
        &(env.ledger().timestamp() + 10000),
        &true,
        &0u32,
    );

    // Stranger cannot add to whitelist
    let result = client.try_add_to_whitelist(&stranger, &project.id, &donor);
    assert!(result.is_err());

    // Admin CAN add to whitelist
    client.add_to_whitelist(&admin, &project.id, &donor);

    // Creator can remove
    client.remove_from_whitelist(&creator, &project.id, &donor);
}

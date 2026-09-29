#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

fn setup() -> (Env, Address, RbacAccessClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let super_admin = Address::generate(&env);
    let contract_id = env.register(RbacAccess, (&super_admin,));
    let client = RbacAccessClient::new(&env, &contract_id);
    (env, super_admin, client)
}

// ── Issue #1133: role_rank total-ordering test ──────────────────────────────

/// Assert that role_rank is strictly increasing in the intended order:
/// ReadOnly < ComplianceAdmin < OperationsAdmin < SuperAdmin.
/// A future change to the Role enum or role_rank match that breaks this
/// ordering will fail here rather than shipping silently.
#[test]
fn test_role_rank_is_strictly_increasing() {
    let roles_in_order = [
        Role::ReadOnly,
        Role::ComplianceAdmin,
        Role::OperationsAdmin,
        Role::SuperAdmin,
    ];
    for window in roles_in_order.windows(2) {
        let lower = &window[0];
        let higher = &window[1];
        assert!(
            role_rank(lower) < role_rank(higher),
            "role_rank ordering broken: {:?} ({}) should be < {:?} ({})",
            lower,
            role_rank(lower),
            higher,
            role_rank(higher),
        );
    }
}

#[test]
fn test_role_rank_values() {
    assert_eq!(role_rank(&Role::ReadOnly), 1);
    assert_eq!(role_rank(&Role::ComplianceAdmin), 2);
    assert_eq!(role_rank(&Role::OperationsAdmin), 3);
    assert_eq!(role_rank(&Role::SuperAdmin), 4);
}

// ── Issue #1131: transfer_super_admin SuperAdminCount correctness ───────────

/// When new_admin does NOT already hold SuperAdmin, a transfer is 1-for-1:
/// count stays unchanged.
#[test]
fn test_transfer_super_admin_to_new_address_keeps_count() {
    let (env, super_admin, client) = setup();
    let new_admin = Address::generate(&env);

    assert_eq!(client.super_admin_count(), 1);
    client.transfer_super_admin(&super_admin, &new_admin);
    assert_eq!(client.super_admin_count(), 1);
    assert!(!client.has_role(&super_admin, &Role::SuperAdmin));
    assert!(client.has_role(&new_admin, &Role::SuperAdmin));
}

/// When new_admin ALREADY holds SuperAdmin, the transfer merges two distinct
/// super-admin identities into one — count must be decremented.
/// This is the bug fixed by issue #1131.
#[test]
fn test_transfer_super_admin_to_existing_super_admin_decrements_count() {
    let (env, super_admin, client) = setup();
    let second_admin = Address::generate(&env);

    // Grant second_admin SuperAdmin so there are now 2.
    client.grant_role(&super_admin, &second_admin, &Role::SuperAdmin);
    assert_eq!(client.super_admin_count(), 2);

    // Transfer super_admin's role to second_admin (who already has it).
    client.transfer_super_admin(&super_admin, &second_admin);

    // super_admin lost their role; second_admin already had it —
    // net result: 1 distinct SuperAdmin, not 2 (or still 2, which was the bug).
    assert_eq!(client.super_admin_count(), 1);
    assert!(!client.has_role(&super_admin, &Role::SuperAdmin));
    assert!(client.has_role(&second_admin, &Role::SuperAdmin));
}

#[test]
#[should_panic(expected = "transfer_super_admin: cannot transfer to self")]
fn test_transfer_super_admin_to_self_panics() {
    let (env, super_admin, client) = setup();
    client.transfer_super_admin(&super_admin, &super_admin);
}

// ── Issue #1132: on-chain role enumeration ───────────────────────────────────

#[test]
fn test_list_role_holders_empty_initially() {
    let (_env, _super_admin, client) = setup();
    // ReadOnly starts empty
    assert_eq!(client.role_holder_count(&Role::ReadOnly), 0);
}

#[test]
fn test_list_role_holders_reflects_grants() {
    let (env, super_admin, client) = setup();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.grant_role(&super_admin, &alice, &Role::ComplianceAdmin);
    client.grant_role(&super_admin, &bob, &Role::ComplianceAdmin);

    assert_eq!(client.role_holder_count(&Role::ComplianceAdmin), 2);
    let page = client.list_role_holders(&Role::ComplianceAdmin, &0, &50);
    assert_eq!(page.len(), 2);
}

#[test]
fn test_list_role_holders_removed_after_revoke() {
    let (env, super_admin, client) = setup();
    let alice = Address::generate(&env);

    client.grant_role(&super_admin, &alice, &Role::OperationsAdmin);
    assert_eq!(client.role_holder_count(&Role::OperationsAdmin), 1);

    client.revoke_role(&super_admin, &alice, &Role::OperationsAdmin);
    assert_eq!(client.role_holder_count(&Role::OperationsAdmin), 0);
}

#[test]
fn test_list_role_holders_pagination() {
    let (env, super_admin, client) = setup();
    for _ in 0..5 {
        let addr = Address::generate(&env);
        client.grant_role(&super_admin, &addr, &Role::ReadOnly);
    }
    assert_eq!(client.role_holder_count(&Role::ReadOnly), 5);

    let page1 = client.list_role_holders(&Role::ReadOnly, &0, &3);
    assert_eq!(page1.len(), 3);

    let page2 = client.list_role_holders(&Role::ReadOnly, &3, &3);
    assert_eq!(page2.len(), 2);
}

#[test]
fn test_list_role_holders_idempotent_grant() {
    let (env, super_admin, client) = setup();
    let alice = Address::generate(&env);

    client.grant_role(&super_admin, &alice, &Role::ReadOnly);
    client.grant_role(&super_admin, &alice, &Role::ReadOnly); // second grant is no-op
    assert_eq!(client.role_holder_count(&Role::ReadOnly), 1);
}

// ── General RBAC tests ────────────────────────────────────────────────────────

#[test]
fn test_grant_and_has_role() {
    let (env, super_admin, client) = setup();
    let alice = Address::generate(&env);

    assert!(!client.has_role(&alice, &Role::ReadOnly));
    client.grant_role(&super_admin, &alice, &Role::ReadOnly);
    assert!(client.has_role(&alice, &Role::ReadOnly));
}

#[test]
fn test_revoke_role() {
    let (env, super_admin, client) = setup();
    let alice = Address::generate(&env);

    client.grant_role(&super_admin, &alice, &Role::ReadOnly);
    client.revoke_role(&super_admin, &alice, &Role::ReadOnly);
    assert!(!client.has_role(&alice, &Role::ReadOnly));
}

#[test]
#[should_panic(expected = "revoke_role: cannot revoke the last SuperAdmin")]
fn test_cannot_revoke_last_super_admin() {
    let (env, super_admin, client) = setup();
    client.revoke_role(&super_admin, &super_admin, &Role::SuperAdmin);
}

#[test]
fn test_super_admin_count_increments_on_grant() {
    let (env, super_admin, client) = setup();
    let second = Address::generate(&env);

    assert_eq!(client.super_admin_count(), 1);
    client.grant_role(&super_admin, &second, &Role::SuperAdmin);
    assert_eq!(client.super_admin_count(), 2);
}

#[test]
fn test_super_admin_count_decrements_on_revoke() {
    let (env, super_admin, client) = setup();
    let second = Address::generate(&env);

    client.grant_role(&super_admin, &second, &Role::SuperAdmin);
    assert_eq!(client.super_admin_count(), 2);
    client.revoke_role(&super_admin, &second, &Role::SuperAdmin);
    assert_eq!(client.super_admin_count(), 1);
}

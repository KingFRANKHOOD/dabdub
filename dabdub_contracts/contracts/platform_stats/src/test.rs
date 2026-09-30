#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env};

fn setup() -> (Env, PlatformStatsContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let id = env.register(PlatformStatsContract, (&admin,));
    let client = PlatformStatsContractClient::new(&env, &id);
    (env, client, admin)
}

#[test]
fn test_initial_stats_are_zero() {
    let (env, client, _admin) = setup();
    let stats = client.stats();
    assert_eq!(stats.total_merchants, 0);
    assert_eq!(stats.total_payments, 0);
    assert_eq!(stats.total_settled_volume_usd, 0);
    assert_eq!(stats.active_payments_24h, 0);
    assert!(stats.health.storage_ok);
    assert!(stats.health.stellar_ok);
    assert!(stats.health.partner_ok);
}

#[test]
fn test_record_merchant_and_payment() {
    let (_env, client, admin) = setup();
    client.record_merchant(&admin);
    client.record_payment(&admin, &10_000_000i128, &true);
    client.record_payment(&admin, &5_000_000i128, &true);
    client.record_payment(&admin, &7_000_000i128, &false);

    let stats = client.stats();
    assert_eq!(stats.total_merchants, 1);
    assert_eq!(stats.total_payments, 3);
    assert_eq!(stats.total_settled_volume_usd, 15_000_000);
    assert_eq!(stats.active_payments_24h, 3);
}

#[test]
#[should_panic(expected = "not admin")]
fn test_record_merchant_requires_admin() {
    let (env, client, _admin) = setup();
    let attacker = Address::generate(&env);
    client.record_merchant(&attacker);
}

#[test]
#[should_panic(expected = "not admin")]
fn test_record_payment_requires_admin() {
    let (env, client, _admin) = setup();
    let attacker = Address::generate(&env);
    client.record_payment(&attacker, &100i128, &true);
}

#[test]
fn test_partner_health_status_toggle() {
    let (env, client, admin) = setup();
    assert!(client.stats().health.partner_ok);
    client.set_partner_ok(&admin, &false);
    assert!(!client.stats().health.partner_ok);
}

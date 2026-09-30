#![no_std]

mod test;

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

/// Number of ledgers in a ~24h window (5s per ledger).
const ACTIVE_WINDOW_LEDGERS: u32 = 17_280;

/// Live platform overview metrics for the admin dashboard.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PlatformStats {
    pub total_merchants: u32,
    pub total_payments: u32,
    pub total_settled_volume_usd: i128,
    pub active_payments_24h: u32,
    pub health: SystemHealth,
}

/// System health: DB/storage, Stellar connectivity, partner API.
#[contracttype]
#[derive(Clone, Debug)]
pub struct SystemHealth {
    pub storage_ok: bool,
    pub stellar_ok: bool,
    pub partner_ok: bool,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    TotalMerchants,
    TotalPayments,
    TotalSettledVolumeUsd,
    /// Rolling count of payments in the current 24h bucket.
    ActiveBucket(u32),
    PartnerOk,
}

#[contract]
pub struct PlatformStatsContract;

#[contractimpl]
impl PlatformStatsContract {
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TotalMerchants, &0u32);
        env.storage().instance().set(&DataKey::TotalPayments, &0u32);
        env.storage().instance().set(&DataKey::TotalSettledVolumeUsd, &0i128);
        env.storage().instance().set(&DataKey::PartnerOk, &true);
    }

    /// Admin-only: record a newly registered merchant.
    pub fn record_merchant(env: Env, caller: Address) {
        caller.require_auth();
        Self::require_admin(&env, &caller);
        let prev: u32 = env.storage().instance().get(&DataKey::TotalMerchants).unwrap();
        env.storage().instance().set(&DataKey::TotalMerchants, &(prev + 1));
    }

    /// Admin-only: record a payment. If `settled`, adds to settled USD volume
    /// and increments the rolling 24h active-payments bucket.
    pub fn record_payment(env: Env, caller: Address, amount_usd: i128, settled: bool) {
        caller.require_auth();
        Self::require_admin(&env, &caller);

        let tp: u32 = env.storage().instance().get(&DataKey::TotalPayments).unwrap();
        env.storage().instance().set(&DataKey::TotalPayments, &(tp + 1));

        if settled {
            let vol: i128 = env
                .storage()
                .instance()
                .get(&DataKey::TotalSettledVolumeUsd)
                .unwrap();
            env.storage()
                .instance()
                .set(&DataKey::TotalSettledVolumeUsd, &(vol + amount_usd));
        }

        let bucket = env.ledger().sequence() / ACTIVE_WINDOW_LEDGERS;
        let key = DataKey::ActiveBucket(bucket);
        let count: u32 = env.storage().instance().get(&key).unwrap_or(0);
        env.storage().instance().set(&key, &(count + 1));
    }

    /// Admin-only: reflect partner API health status.
    pub fn set_partner_ok(env: Env, caller: Address, ok: bool) {
        caller.require_auth();
        Self::require_admin(&env, &caller);
        env.storage().instance().set(&DataKey::PartnerOk, &ok);
    }

    /// Live overview metrics, computed directly from storage.
    pub fn stats(env: Env) -> PlatformStats {
        let bucket = env.ledger().sequence() / ACTIVE_WINDOW_LEDGERS;
        let active: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ActiveBucket(bucket))
            .unwrap_or(0);

        PlatformStats {
            total_merchants: env
                .storage()
                .instance()
                .get(&DataKey::TotalMerchants)
                .unwrap_or(0),
            total_payments: env
                .storage()
                .instance()
                .get(&DataKey::TotalPayments)
                .unwrap_or(0),
            total_settled_volume_usd: env
                .storage()
                .instance()
                .get(&DataKey::TotalSettledVolumeUsd)
                .unwrap_or(0),
            active_payments_24h: active,
            health: Self::health(&env),
        }
    }

    fn require_admin(env: &Env, caller: &Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if caller != &admin {
            panic!("not admin");
        }
    }

    fn health(env: &Env) -> SystemHealth {
        SystemHealth {
            storage_ok: env.storage().instance().has(&DataKey::Admin),
            stellar_ok: env.ledger().sequence() > 0,
            partner_ok: env
                .storage()
                .instance()
                .get(&DataKey::PartnerOk)
                .unwrap_or(true),
        }
    }
}

#![no_std]

mod test;

use soroban_sdk::{
    contract, contractevent, contractimpl, contracttype, Address, Env, Vec,
};

// ── Role ────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    ReadOnly,
    ComplianceAdmin,
    OperationsAdmin,
    SuperAdmin,
}

/// Strict total order used by `require_role` to enforce minimum-rank checks.
/// ReadOnly=1, ComplianceAdmin=2, OperationsAdmin=3, SuperAdmin=4.
pub fn role_rank(role: &Role) -> u32 {
    match role {
        Role::ReadOnly => 1,
        Role::ComplianceAdmin => 2,
        Role::OperationsAdmin => 3,
        Role::SuperAdmin => 4,
    }
}

// ── Storage keys ────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Whether the account holds the given role
    AccountRole(Address, Role),
    /// Count of distinct addresses currently holding SuperAdmin
    SuperAdminCount,
    /// Index: list of all addresses currently holding a given role
    RoleHolders(Role),
}

// ── Events ───────────────────────────────────────────────────────────────────

#[contractevent(topics = ["RBAC", "role_granted"])]
pub struct RoleGrantedEvent {
    pub account: Address,
    pub role: Role,
}

#[contractevent(topics = ["RBAC", "role_revoked"])]
pub struct RoleRevokedEvent {
    pub account: Address,
    pub role: Role,
}

#[contractevent(topics = ["RBAC", "super_admin_transferred"])]
pub struct SuperAdminTransferredEvent {
    pub from: Address,
    pub to: Address,
}

// ── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct RbacAccess;

#[contractimpl]
impl RbacAccess {
    // ── Constructor ───────────────────────────────────────────────────────────

    pub fn __constructor(env: Env, initial_super_admin: Address) {
        env.storage()
            .instance()
            .set(&DataKey::SuperAdminCount, &1u32);
        env.storage().instance().set(
            &DataKey::AccountRole(initial_super_admin.clone(), Role::SuperAdmin),
            &true,
        );
        let mut holders: Vec<Address> = Vec::new(&env);
        holders.push_back(initial_super_admin.clone());
        env.storage()
            .instance()
            .set(&DataKey::RoleHolders(Role::SuperAdmin), &holders);
        RoleGrantedEvent {
            account: initial_super_admin,
            role: Role::SuperAdmin,
        }
        .publish(&env);
    }

    // ── Role management ───────────────────────────────────────────────────────

    /// Grant `role` to `account`. Caller must hold `SuperAdmin`.
    pub fn grant_role(env: Env, caller: Address, account: Address, role: Role) {
        caller.require_auth();
        Self::require_role_internal(&env, &caller, &Role::SuperAdmin);

        let key = DataKey::AccountRole(account.clone(), role.clone());
        if env.storage().instance().get::<_, bool>(&key).unwrap_or(false) {
            return; // idempotent
        }

        env.storage().instance().set(&key, &true);

        if role == Role::SuperAdmin {
            let count: u32 = env
                .storage()
                .instance()
                .get(&DataKey::SuperAdminCount)
                .unwrap_or(0);
            env.storage()
                .instance()
                .set(&DataKey::SuperAdminCount, &(count + 1));
        }

        // Update role-holders index
        Self::index_add(&env, &account, &role);

        RoleGrantedEvent {
            account,
            role,
        }
        .publish(&env);
    }

    /// Revoke `role` from `account`. Caller must hold `SuperAdmin`.
    /// Panics if this would remove the last SuperAdmin.
    pub fn revoke_role(env: Env, caller: Address, account: Address, role: Role) {
        caller.require_auth();
        Self::require_role_internal(&env, &caller, &Role::SuperAdmin);

        let key = DataKey::AccountRole(account.clone(), role.clone());
        if !env.storage().instance().get::<_, bool>(&key).unwrap_or(false) {
            return; // idempotent
        }

        if role == Role::SuperAdmin {
            let count: u32 = env
                .storage()
                .instance()
                .get(&DataKey::SuperAdminCount)
                .unwrap_or(0);
            if count <= 1 {
                panic!("revoke_role: cannot revoke the last SuperAdmin");
            }
            env.storage()
                .instance()
                .set(&DataKey::SuperAdminCount, &(count - 1));
        }

        env.storage().instance().remove(&key);
        Self::index_remove(&env, &account, &role);

        RoleRevokedEvent {
            account,
            role,
        }
        .publish(&env);
    }

    /// Transfer the caller's SuperAdmin role to `new_admin`.
    ///
    /// Fix for issue #1131: if `new_admin` already independently holds
    /// SuperAdmin, the transfer merges two distinct identities into one.
    /// SuperAdminCount is decremented to reflect the loss of a distinct
    /// super-admin identity (the caller's slot is vacated without a
    /// corresponding new slot being created).
    pub fn transfer_super_admin(env: Env, caller: Address, new_admin: Address) {
        caller.require_auth();
        Self::require_role_internal(&env, &caller, &Role::SuperAdmin);

        if caller == new_admin {
            panic!("transfer_super_admin: cannot transfer to self");
        }

        let new_admin_already_super = env
            .storage()
            .instance()
            .get::<_, bool>(&DataKey::AccountRole(new_admin.clone(), Role::SuperAdmin))
            .unwrap_or(false);

        // Revoke caller's SuperAdmin
        env.storage()
            .instance()
            .remove(&DataKey::AccountRole(caller.clone(), Role::SuperAdmin));
        Self::index_remove(&env, &caller, &Role::SuperAdmin);

        if new_admin_already_super {
            // Merging two SuperAdmin identities into one — decrement count.
            let count: u32 = env
                .storage()
                .instance()
                .get(&DataKey::SuperAdminCount)
                .unwrap_or(1);
            env.storage()
                .instance()
                .set(&DataKey::SuperAdminCount, &count.saturating_sub(1));
        } else {
            // new_admin gains SuperAdmin — count stays the same (one in, one out).
            env.storage().instance().set(
                &DataKey::AccountRole(new_admin.clone(), Role::SuperAdmin),
                &true,
            );
            Self::index_add(&env, &new_admin, &Role::SuperAdmin);
        }

        SuperAdminTransferredEvent {
            from: caller,
            to: new_admin,
        }
        .publish(&env);
    }

    // ── Queries ────────────────────────────────────────────────────────────────

    pub fn has_role(env: Env, account: Address, role: Role) -> bool {
        env.storage()
            .instance()
            .get::<_, bool>(&DataKey::AccountRole(account, role))
            .unwrap_or(false)
    }

    pub fn super_admin_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::SuperAdminCount)
            .unwrap_or(0)
    }

    /// Return a page of addresses holding `role`.
    ///
    /// Implements issue #1132: on-chain enumeration of role holders.
    /// `offset` is the 0-based start index; `limit` caps the page size (max 50).
    pub fn list_role_holders(env: Env, role: Role, offset: u32, limit: u32) -> Vec<Address> {
        let cap = limit.min(50);
        let holders: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::RoleHolders(role))
            .unwrap_or(Vec::new(&env));

        let mut page: Vec<Address> = Vec::new(&env);
        let len = holders.len();
        let mut i = offset;
        while i < len && i < offset + cap {
            page.push_back(holders.get(i).unwrap());
            i += 1;
        }
        page
    }

    /// Return the total number of addresses holding `role`.
    pub fn role_holder_count(env: Env, role: Role) -> u32 {
        let holders: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::RoleHolders(role))
            .unwrap_or(Vec::new(&env));
        holders.len()
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn require_role_internal(env: &Env, account: &Address, required: &Role) {
        let has = env
            .storage()
            .instance()
            .get::<_, bool>(&DataKey::AccountRole(account.clone(), required.clone()))
            .unwrap_or(false);
        if !has {
            panic!("rbac: missing required role");
        }
    }

    fn index_add(env: &Env, account: &Address, role: &Role) {
        let mut holders: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::RoleHolders(role.clone()))
            .unwrap_or(Vec::new(env));
        // Avoid duplicates
        for h in holders.iter() {
            if h == *account {
                return;
            }
        }
        holders.push_back(account.clone());
        env.storage()
            .instance()
            .set(&DataKey::RoleHolders(role.clone()), &holders);
    }

    fn index_remove(env: &Env, account: &Address, role: &Role) {
        let holders: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::RoleHolders(role.clone()))
            .unwrap_or(Vec::new(env));
        let mut updated: Vec<Address> = Vec::new(env);
        for h in holders.iter() {
            if h != *account {
                updated.push_back(h);
            }
        }
        env.storage()
            .instance()
            .set(&DataKey::RoleHolders(role.clone()), &updated);
    }
}

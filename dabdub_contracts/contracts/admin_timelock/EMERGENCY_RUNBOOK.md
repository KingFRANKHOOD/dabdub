# Emergency Runbook: admin_timelock

## Overview
The `admin_timelock` contract gates all privileged parameter changes across the Stellar platform.

## Incident: Malicious/Erroneous Scheduled Change

### Symptoms
- A scheduled change has been proposed that would lock out users or drain funds
- The change is scheduled but not yet executed

### Response Steps

1. **Verify the pending change**
```bash
soroban contract invoke \
  --id ADMIN_TIMELOCK_CONTRACT_ID \
  -- get_pending_changes
```

2. **Assess urgency** - check time until `execute_after`

3. **Initiate cancellation** - if admin key available:
```bash
soroban contract invoke \
  --id ADMIN_TIMELOCK_CONTRACT_ID \
  --source ADMIN_KEYPAIR \
  -- cancel_change \
  --change_id CHANGE_ID
```

4. **Verify cancellation** - re-query to confirm

### Prevention
- Keep admin key in secure hardware wallet
- Require multisig for admin operations where possible
- Monitor all scheduled changes via event logs
- Use short `ledgers_to_lock` values (1-2 hours)

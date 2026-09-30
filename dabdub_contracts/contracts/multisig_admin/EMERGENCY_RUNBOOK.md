# Emergency Runbook: multisig_admin

## Overview
The `multisig_admin` contract implements M-of-N signature-based governance.

## Incident: Lost Signer Key

### Symptoms
- A multisig signer has lost access to their key
- Transactions requiring M signatures lack sufficient signers

### Recovery

Check current signers:
```bash
soroban contract invoke \
  --id MULTISIG_ADMIN_CONTRACT_ID \
  -- get_admin_signers
```

**Note:** Current contract lacks remove_admin/replace_admin function.

### Options
1. Multisig Governance Vote - Remaining M signers propose rotation
2. Emergency Multi-Signature Update - Gather all available signers
3. Contract Upgrade (Last Resort) - Deploy new multisig with updated signers

### Prevention
- Use hardware wallets for all signers
- Maintain geographically distributed signing authority
- Conduct regular key rotation
- Maintain detailed audit logs

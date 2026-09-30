# Emergency Runbook: settlement_ledger

## Overview
The `settlement_ledger` is the permanent on-chain audit trail of fiat settlements.

## Incident: Erroneous Settlement Record

### Symptoms
- A settlement record was written for a payment_id that was never released
- The amount or merchant doesn't match the escrow payment
- Records are immutable, cannot be "fixed" in-place

### Recovery

1. **Identify the erroneous record**
```bash
soroban contract invoke \
  --id SETTLEMENT_LEDGER_CONTRACT_ID \
  -- get_settlement \
  --payment_id PAYMENT_ID
```

2. **Verify against escrow**
```bash
soroban contract invoke \
  --id PAYMENT_ESCROW_CONTRACT_ID \
  -- get_payment \
  --payment_id PAYMENT_ID
```

3. **Create correcting settlement record** with offsetting amounts to reconcile

### Prevention
- Configure PaymentEscrow contract for cross-validation
- Embrace immutability - create correcting entries, not patches
- Clear fiat_ref values for offline reconciliation
- Keep admin key secure
- Reconcile settlement ledger against fiat processor daily

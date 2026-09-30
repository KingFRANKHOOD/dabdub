## Summary

Implements four features across the auth and waitlist modules.

### Issues resolved

- Closes dupdab/dupdapp_stellar#589 — `POST /api/v1/auth/login`: bcrypt password comparison, JWT signing with configurable secret/expiry, returns `{ accessToken, refreshToken, merchant }`
- Closes dupdab/dupdapp_stellar#590 — `POST /api/v1/auth/refresh`: refresh token rotation, SHA-256 hash stored in DB, expired/used tokens rejected with 401
- Closes dupdab/dupdapp_stellar#592 — `GET /api/v1/auth/verify-email?token=xxx`: sets `emailVerified` flag, 24-hour expiry, `POST /api/v1/auth/resend-verification` endpoint
- Closes dupdab/dupdapp_stellar#690 — `POST /api/v1/waitlist/join`: generates unique referral code per member, accepts `referralCode` on join, moves referrer up 5 queue positions atomically, rejects self-referrals with 400

### Files added

```
backend/
  src/
    app.module.ts
    auth/
      auth.controller.ts     — login, refresh, verify-email, resend-verification
      auth.module.ts
      auth.service.ts        — all auth business logic
      refresh-token.entity.ts
    merchant/
      merchant.entity.ts     — emailVerified, emailVerifyToken, emailVerifyExpiry
    waitlist/
      waitlist.controller.ts
      waitlist.entity.ts     — referralCode, referralCount, position
      waitlist.module.ts
      waitlist.service.ts    — join + referral position logic (atomic transaction)
  package.json
  tsconfig.json
```

### Key implementation notes

- Refresh tokens stored as SHA-256 hashes — raw token never persisted
- Rotation enforced: each use of a refresh token deletes the old one and issues a new one
- Referral position shift runs inside a TypeORM transaction to prevent race conditions
- Email verification expiry is 24 hours; resend endpoint resets the token and expiry
- JWT payload includes `sub` (merchantId) and `email` per spec
- No new dependencies beyond what a standard NestJS/TypeORM project already uses

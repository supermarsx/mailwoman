# Upgrade note: password policy and Assist settings from the console

Applies to every deployment. No action is required, but values saved in the admin console before the upgrade start to apply after it. Check the values below before you upgrade.

## Password policy (`passwd.policy.*`)

**Before:** the five password policy settings (minimum length, and whether an upper-case letter, a lower-case letter, a digit or a symbol is required) could be saved in the console, but nothing read them. Every password change used a fixed policy of at least 12 characters with no character classes, whatever the console said.

**After:**

- The policy is the deployment value, or the domain's value where one is set. `GET /api/password/policy` (the policy the web app shows before a change) reports it, and `POST /api/password` and `mailwoman password` enforce it on the new password.
- A minimum length lower than 12 is honoured. The lowest minimum is 8: the console does not accept a lower value, and every password backend keeps 8 as its own floor.
- The policy applies only when a password is changed. A password set before the upgrade, or before a stricter policy was saved, still signs in. No account is locked out or asked to change its password because of the policy. To make people choose a password that meets a new policy, set **Force password change** on their accounts.
- With no value saved, the policy is the registry default: at least 12 characters, no character classes. That is the same as before the upgrade.
- There is no administrator "set a user's mail password" route; mail passwords are changed by the account holder or with `mailwoman password`. The admin console's own sign-in passwords are separate and keep their own rule (12 to 1024 characters).

To check what will apply, look up the `passwd.policy.*` keys under **Settings** in the admin console (or `GET /admin/settings/passwd.policy.minLength` and the other four keys) and review the deployment value and any domain values.

## Assist settings

- **`assistant.enabled`** (deployment, domain or user) now turns Assist off where it is `false`. It sits below the deployment switch and kill switch on the Assist page: it can turn Assist off, not on.
- **`limit.ai_calls_per_day`** (deployment, domain or per user) now limits the Assist requests (`/api/assist/invoke` and `/api/assist/transcribe`) a person can make per UTC day. `0` (the default) means no limit. A request over the limit is answered `429` with `code: "limit.ai_calls_per_day"`, and nothing is sent to the AI endpoint. A request the gateway refuses, or one that fails before the endpoint replies, is not counted. Semantic search re-ranking is not counted by this limit; `assistant.rateLimitPerMin` bounds it.
- **`assistant.rateLimitPerMin`** is now read from the console. `MW_ASSIST_RATE_LIMIT_PER_MIN`, when set, still wins over the console value. The value is read when the server starts, so a change applies after a restart. **`0` now means no limit**, as the console describes the setting; before the upgrade, `MW_ASSIST_RATE_LIMIT_PER_MIN=0` refused every Assist request. If you used `0` to stop Assist, use the kill switch on the Assist page instead.

## Plugin storage limits (`plugins.kv.*`)

The four plugin storage limits (key size, value size, total size and number of keys per plugin and account) are now read from the console at every write. Each environment variable (`MW_PLUGIN_KV_MAX_KEY_BYTES`, `MW_PLUGIN_KV_MAX_VALUE_BYTES`, `MW_PLUGIN_KV_MAX_TOTAL_BYTES`, `MW_PLUGIN_KV_MAX_KEYS`), when set, still wins over the console value. Data already stored above a lowered limit is kept; the limit applies to the next write.

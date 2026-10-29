-- t77: `twofa.lastFactorRemoval = requireRecoveryCodes` (t39-e6) only asked for
-- recovery codes before the last passkey or TOTP was removed, after which the
-- account signed in with the password alone. It now means `block`: the last
-- factor stays until another one is added. Stored values are rewritten so the
-- admin console shows the value that applies. The server also reads the old
-- value as `block`, and the registry accepts it on write as `block`.
UPDATE setting_policy
   SET global_value = '"block"'
 WHERE key = 'twofa.lastFactorRemoval'
   AND global_value = '"requireRecoveryCodes"';
UPDATE setting_value
   SET value = '"block"'
 WHERE key = 'twofa.lastFactorRemoval'
   AND value = '"requireRecoveryCodes"';

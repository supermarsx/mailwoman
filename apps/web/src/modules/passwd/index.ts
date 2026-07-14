// V7 in-app password-change module (SPEC §18.3, plan §2.6 / §3 e7). Lazily importable;
// NOT routed by this module (ownership boundary — e14 mounts it into Settings' security
// section; PREFER exporting over editing Settings.tsx to avoid colliding with e6).
//
// e14 WIRE-UP (import path):
//   import { PasswordChange } from './modules/passwd/index.ts'
//   // plain account:
//   <PasswordChange accountId={id} />
//   // zero-access account (re-wrap; recovery phrase shown BEFORE the change):
//   <PasswordChange accountId={id} zeroAccess={{ account, za: spawnZeroAccessWorker() }} />
// Endpoints this module calls (e9 to satisfy, e14 to mount):
//   GET  /api/password/policy   → PasswordPolicy
//   POST /api/password (PasswordChangeRequest) → PasswordChangeOutcome
//     (rewrap material is present only for zero-access, only AFTER the pre-prompt)

export { PasswordChange, type PasswordChangeProps } from './PasswordChange.tsx';
export {
  PasswordService,
  policyViolations,
  type Fetcher,
  type PasswordPolicy,
  type PasswordChangeRequest,
  type PasswordChangeOutcome,
  type RewrapPayload,
} from './service.ts';
export {
  recoveryPhraseBefore,
  rewrapUnderNewPassword,
  type RewrapResult,
  type RewrapInputs,
} from './rewrap.ts';

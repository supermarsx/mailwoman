// Settings-surface shared types (t16 e15, SPEC §7.4/§19 + W12/W13/W15).
//
// These mirror the JSON the server contracts return. The 2FA + session shapes
// track `crates/mw-server/src/twofa_routes.rs`; the prefs shapes (signatures,
// notification rules, saved searches, identities) mirror the `mw-store` 0017 /
// frozen-0003 rows they persist. Kept transport-agnostic so the components and
// their unit tests share one vocabulary.

// ── 2FA (S1) ─────────────────────────────────────────────────────────────────

/** A registered passkey as the account status endpoint reports it (no secret). */
export interface PasskeySummary {
  /** Stable non-secret handle used to address the credential for removal. */
  readonly handle: string;
  readonly label: string;
  readonly createdAt: string;
}

/** `GET /api/account/2fa` — the caller's second-factor status. */
export interface TwofaStatus {
  /** A confirmed TOTP secret is enrolled. */
  readonly totp: boolean;
  readonly passkeys: readonly PasskeySummary[];
  /** Count of unused recovery codes remaining. */
  readonly recoveryRemaining: number;
  /** An admin policy (global/domain) requires a second factor for this account. */
  readonly policyRequired: boolean;
}

/** `POST /api/account/2fa/totp/begin` — an unconfirmed TOTP secret to display. */
export interface TotpBegin {
  /** Base32 secret for manual authenticator entry. */
  readonly secret: string;
  /** `otpauth://` provisioning URI (drives a QR the app can render). */
  readonly otpauthUri: string;
}

/** A passkey registration challenge (`POST /api/account/2fa/passkey/begin`). */
export interface PasskeyRegistrationChallenge {
  /** base64url challenge to feed `navigator.credentials.create`. */
  readonly challenge: string;
  readonly rpId: string;
  /** base64url of the WebAuthn user handle (stable per account). */
  readonly userHandle: string;
  readonly userName: string;
  readonly userVerification: string;
}

/** The one-shot recovery codes an enrolment/regenerate returns (shown ONCE). */
export interface RecoveryCodes {
  readonly recoveryCodes: readonly string[];
}

// ── Sessions (S11) ───────────────────────────────────────────────────────────

/** One active session as `GET /api/account/sessions` reports it (metadata only). */
export interface SessionMeta {
  readonly handle: string;
  readonly username: string;
  readonly createdAt: string;
  readonly lastSeen: string;
  /** True for the session making the request (never revocable from here). */
  readonly current: boolean;
}

// ── Login-time second factor (S1 web half) ───────────────────────────────────

/** The `twofaRequired` body `/api/login` returns when a factor must be cleared. */
export interface LoginChallenge {
  readonly pendingToken: string;
  /** Which factors the user may present ("totp" | "webauthn" | "recovery"). */
  readonly factors: readonly string[];
  /** Present when a policy-required user has nothing enrolled yet. */
  readonly enrollmentRequired?: boolean;
  readonly webauthn?: {
    readonly challenge: string;
    readonly credentialIds: readonly string[];
    readonly rpId: string;
    readonly userVerification: string;
  };
}

// ── Signatures (W12) ─────────────────────────────────────────────────────────

/** A signature template (`mw-store` 0017 `signatures`). */
export interface Signature {
  readonly name: string;
  readonly body: string;
  readonly isDefault: boolean;
  /** Optional JSON rule (e.g. per-identity / per-recipient selection). */
  readonly rule?: string;
}

// ── Identities ───────────────────────────────────────────────────────────────

/** A send identity (maps to JMAP `Identity` server-side). */
export interface Identity {
  readonly id: string;
  readonly name: string;
  readonly email: string;
  readonly replyTo?: string;
  /** Name of the signature template this identity defaults to. */
  readonly signatureName?: string;
}

// ── Notification rules + quiet hours (W15) ───────────────────────────────────

/** A single notification rule (match → notify/mute). */
export interface NotificationRule {
  readonly id: string;
  /** Human label for the rule. */
  readonly label: string;
  /** Match on sender/mailbox/subject substring (any-of). */
  readonly match: string;
  /** "notify" surfaces it; "mute" suppresses notifications for matches. */
  readonly action: 'notify' | 'mute';
}

/** Quiet-hours window (local 24h HH:MM). Suppresses notifications in-range. */
export interface QuietHours {
  readonly enabled: boolean;
  readonly start: string;
  readonly end: string;
}

/** `GET/PUT /api/account/notifications` — the account's notification config. */
export interface NotificationConfig {
  readonly enabled: boolean;
  readonly rules: readonly NotificationRule[];
  readonly quietHours: QuietHours;
}

// ── Saved searches → search folders (W13) ────────────────────────────────────

/** A saved search (frozen `mw-store` 0003 `saved_searches`). */
export interface SavedSearch {
  readonly id: string;
  readonly name: string;
  /** The stored JMAP filter (serialized). */
  readonly queryJson: string;
  /** Surface it as a virtual folder in the mailbox list. */
  readonly asFolder: boolean;
}

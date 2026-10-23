// Telling "this server does not do that" apart from "that failed" (t32-h6).
//
// In proxy mode every JMAP method goes to the upstream server, and a server that
// does not implement a Mailwoman extension (`ServerMetadata/*`, `MailRule/*`, …)
// answers `unknownMethod` (RFC 8620 §3.6.2), or refuses the request outright with
// `unknownCapability` (§3.6.1). Neither is a fault the user can retry away: the
// section that asked should say the feature is not available and carry on.
//
// `responseFor` (api/jmap.ts) reports a method error as `JMAP method error:
// <type> …`, and a request-level problem carries its type URN in the message, so
// the type name is matched in the message.

const UNSUPPORTED = /\bunknown(?:Method|Capability)\b/;

/** True when `err` says the server does not implement the method or capability. */
export function isUnsupportedMethod(err: unknown): boolean {
  const message = err instanceof Error ? err.message : typeof err === 'string' ? err : '';
  return UNSUPPORTED.test(message);
}

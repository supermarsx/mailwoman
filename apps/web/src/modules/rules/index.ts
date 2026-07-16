// Rules (mail-filter) module public surface (audit #1, SPEC §6.1/§10.5).
//
// Mounted into Settings.tsx for an authenticated account:
//   import { RulesSettings } from '../modules/rules/index.ts';
//   <RulesSettings accountId={accountId()} />
//
// The condition/action builder, the raw-Sieve editor (highlight + lint), the
// "where it runs" indicator, and the dry-run all ride the EXISTING `MailRule`
// JMAP surface (`state/slices/rules.ts`) + the server's Sieve codegen/PUTSCRIPT
// path. The web-side Sieve rendering/lint/eval mirrors `crates/mw-sieve` purely
// for preview — the server stays authoritative on the wire.

export { RulesSettings, type RulesSettingsProps } from './RulesSettings.tsx';
export { RuleBuilder } from './RuleBuilder.tsx';
export { RawEditor } from './RawEditor.tsx';
export { WhereItRuns } from './WhereItRuns.tsx';
export { DryRun } from './DryRun.tsx';
export {
  rulesToSieve,
  runsAtFor,
  lintSieve,
  tokenizeSieve,
  dryRun,
  type SampleMessage,
  type DryRunResult,
  type Token,
  type TokenKind,
} from './sieve.ts';

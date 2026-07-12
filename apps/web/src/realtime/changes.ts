// StateChange → refetch reconciliation (plan §2.2, §3 e6).
//
// On a pushed `StateChange` the client must call the matching `*/changes`
// method and refetch — this replaces pure on-demand polling. The engine wiring
// (which `Email/changes` diff feeds which list refetch) lands at integration
// (Batch C); here we own the pure, testable half: given the previous per-type
// state tokens and an incoming `StateChange`, decide which datatypes actually
// moved for an account, so we don't refetch types whose state is unchanged.

import type { StateChange, TypeStates } from '../contracts/push.ts';
import type { Id } from '../api/jmap-types.ts';

/** The JMAP datatypes a `StateChange` can report (§2.2). */
export type ChangedType = 'Email' | 'Mailbox' | 'EmailSubmission' | 'Thread';

const ALL_TYPES: readonly ChangedType[] = ['Email', 'Mailbox', 'EmailSubmission', 'Thread'];

/**
 * Which datatypes moved for `accountId`: a type is "changed" when the pushed
 * state token differs from the last one we acted on (or we had none yet).
 */
export function changedTypes(
  prev: TypeStates | undefined,
  change: StateChange,
  accountId: Id,
): ChangedType[] {
  const next = change.changed[accountId];
  if (next === undefined) return [];
  const out: ChangedType[] = [];
  for (const t of ALL_TYPES) {
    const nextState = next[t];
    if (nextState === undefined) continue;
    if (prev === undefined || prev[t] !== nextState) out.push(t);
  }
  return out;
}

/** Called with the datatypes that moved for an account; wired to the per-type `changes` method. */
export type ReconcileHandler = (accountId: Id, types: ChangedType[]) => void;

export interface ChangeReconciler {
  /** Feed a pushed `StateChange`; invokes the handler for each moved account. */
  apply(change: StateChange): void;
  /** Seed the known state (e.g. from the initial session) without refetching. */
  seed(accountId: Id, states: TypeStates): void;
  /** Forget all remembered state (on logout / account switch). */
  reset(): void;
}

/**
 * Tracks the last per-account/per-type state tokens and, on each `StateChange`,
 * invokes `onChanged` only for the datatypes whose token advanced — the caller
 * then issues the corresponding per-type `changes` method + list refetch.
 */
export function createChangeReconciler(onChanged: ReconcileHandler): ChangeReconciler {
  const known = new Map<Id, TypeStates>();

  function merge(accountId: Id, states: TypeStates): void {
    const prev = known.get(accountId) ?? {};
    known.set(accountId, { ...prev, ...states });
  }

  return {
    apply(change): void {
      for (const accountId of Object.keys(change.changed)) {
        const types = changedTypes(known.get(accountId), change, accountId);
        const states = change.changed[accountId];
        if (states !== undefined) merge(accountId, states);
        if (types.length > 0) onChanged(accountId, types);
      }
    },
    seed(accountId, states): void {
      merge(accountId, states);
    },
    reset(): void {
      known.clear();
    },
  };
}

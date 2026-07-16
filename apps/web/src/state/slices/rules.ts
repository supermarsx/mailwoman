// Rules (mail-filter) store slice (audit #1, SPEC §6.1/§10.5). Owns the
// `MailRule/*` JMAP surface for the web client: list, create/update (via
// `MailRule/set create|update`), delete, and enable/disable toggling. Disjoint
// file — no `store.ts` collision with the other slices (same discipline as the
// V2/V3/V4 slices).
//
// The server persists rules as `mw_sieve::Rule`, projects them to the frozen
// §2.1 `MailRule` DTO on `MailRule/get`, and (where the backend advertises
// ManageSieve) uploads the generated Sieve on `MailRule/set` — so this slice
// rides the EXISTING codegen/PUTSCRIPT path; it never generates Sieve for the
// wire. The web-side Sieve rendering (`../modules/rules/sieve.ts`) is only for
// the raw-editor preview and the dry-run — the server remains authoritative.

import { createSignal, type Accessor } from 'solid-js';
import { CAP_CORE, type Id } from '../../api/jmap-types.ts';
import { responseFor } from '../../api/jmap.ts';
import { CAP_SECURITY, type MailRule } from '../../api/crypto-types.ts';
import type { SliceContext } from './context.ts';

const RULES_USING = [CAP_CORE, CAP_SECURITY];

/** The editable shape of a rule (a `MailRule` without the server-assigned id). */
export type RuleDraft = Omit<MailRule, 'id'> & { id?: Id };

interface RuleGetResponse {
  accountId: Id;
  state: string;
  list: MailRule[];
  notFound: Id[];
}

interface RuleSetResponse {
  accountId: Id;
  created: Record<string, { id: Id } & Partial<MailRule>> | null;
  updated: Record<string, unknown> | null;
  destroyed: Id[] | null;
}

/** A blank rule for the builder's "new rule" flow. */
export function emptyRuleDraft(): RuleDraft {
  return {
    name: '',
    matchAll: true,
    conditions: [{ type: 'from', op: 'contains', value: '' }],
    actions: [{ type: 'move', value: '' }],
    enabled: true,
    runsAt: 'engine',
  };
}

/** The public interface of the rules slice. */
export interface RulesSlice {
  rules: Accessor<MailRule[]>;
  rulesLoading: Accessor<boolean>;
  /** Load the account's mail rules (`MailRule/query` is not needed — get returns all). */
  loadRules(): Promise<void>;
  /** Create (no id) or update (with id) a rule; refreshes the local list. */
  saveRule(draft: RuleDraft): Promise<void>;
  /** Delete a rule by id. */
  deleteRule(id: Id): Promise<void>;
  /** Flip a rule's `enabled` flag. */
  toggleRule(id: Id, enabled: boolean): Promise<void>;
}

export function createRulesSlice(ctx: SliceContext): RulesSlice {
  const client = ctx.client;
  const [rules, setRules] = createSignal<MailRule[]>([]);
  const [rulesLoading, setRulesLoading] = createSignal(false);
  const [accountId, setAccountId] = createSignal<string | null>(null);

  /** Resolve (and cache) the security account id; `null` when none is available. */
  async function resolveAccount(): Promise<string | null> {
    const cur = accountId();
    if (cur !== null) return cur;
    const session = await client.session();
    const primary = session.primaryAccounts[CAP_SECURITY];
    const acct = primary ?? Object.keys(session.accounts)[0] ?? null;
    setAccountId(acct);
    return acct;
  }

  async function loadRules(): Promise<void> {
    setRulesLoading(true);
    try {
      const acct = await resolveAccount();
      if (acct === null) {
        setRules([]);
        return;
      }
      const res = await client.jmap({
        using: RULES_USING,
        methodCalls: [['MailRule/get', { accountId: acct }, 'g']],
      });
      const got = responseFor<RuleGetResponse>(res, 'g');
      setRules(got.list);
    } finally {
      setRulesLoading(false);
    }
  }

  /** The wire body of a rule (drops the id — create omits it, update keys on it). */
  function ruleBody(draft: RuleDraft): Record<string, unknown> {
    return {
      name: draft.name,
      matchAll: draft.matchAll,
      conditions: draft.conditions,
      actions: draft.actions,
      enabled: draft.enabled,
      runsAt: draft.runsAt,
    };
  }

  async function saveRule(draft: RuleDraft): Promise<void> {
    const acct = await resolveAccount();
    if (acct === null) throw new Error('no account available for rules');
    const method =
      draft.id === undefined || draft.id === ''
        ? { create: { new: ruleBody(draft) } }
        : { update: { [draft.id]: ruleBody(draft) } };
    const res = await client.jmap({
      using: RULES_USING,
      methodCalls: [['MailRule/set', { accountId: acct, ...method }, 's']],
    });
    responseFor<RuleSetResponse>(res, 's'); // throws on a method-level error
    await loadRules();
    ctx.broadcastChange?.();
    ctx.showToast('success', draft.id ? 'Rule updated' : 'Rule created');
  }

  async function deleteRule(id: Id): Promise<void> {
    const acct = await resolveAccount();
    if (acct === null) throw new Error('no account available for rules');
    const res = await client.jmap({
      using: RULES_USING,
      methodCalls: [['MailRule/set', { accountId: acct, destroy: [id] }, 's']],
    });
    responseFor<RuleSetResponse>(res, 's');
    setRules(rules().filter((r) => r.id !== id));
    ctx.broadcastChange?.();
    ctx.showToast('success', 'Rule deleted');
  }

  async function toggleRule(id: Id, enabled: boolean): Promise<void> {
    const acct = await resolveAccount();
    if (acct === null) throw new Error('no account available for rules');
    // Optimistic local flip; the reload reconciles.
    setRules(rules().map((r) => (r.id === id ? { ...r, enabled } : r)));
    const res = await client.jmap({
      using: RULES_USING,
      methodCalls: [['MailRule/set', { accountId: acct, update: { [id]: { enabled } } }, 's']],
    });
    responseFor<RuleSetResponse>(res, 's');
    ctx.broadcastChange?.();
  }

  return { rules, rulesLoading, loadRules, saveRule, deleteRule, toggleRule };
}

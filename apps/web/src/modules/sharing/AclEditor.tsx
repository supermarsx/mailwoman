// Mailbox ACL editor (RFC 4314) — t13 e8, plan §Workstream-2 E8.
//
// Lists each identifier's grant on a mailbox as a fieldset of the eleven RFC 4314
// rights checkboxes (labelled, plain-language), shows the current user's MYRIGHTS,
// and — ONLY when the current user holds the `a` (administer) right — exposes the
// add/toggle/remove (SETACL/DELETEACL) affordances. Without `a` the editor is
// strictly read-only: the checkboxes are disabled and no write control renders.
// The upstream IMAP server is the real enforcer; this gate is UX honesty.
//
// Self-contained + EXPORTED (not router-mounted): E9 mounts it and injects the
// production `AclClient` (`createAclClient(accountId, client.jmap)`).

import { createResource, createSignal, For, Show, onMount, createMemo, type JSX } from 'solid-js';
import { t, loadCatalog, isolate } from '../../i18n';
import {
  ACL_RIGHTS,
  canAdminister,
  hasRight,
  serializeRights,
  toggleRight,
  type AclClient,
  type AclEntry,
  type AclRight,
  type MailboxRights,
} from '../../api/acl-types.ts';
import type { Id } from '../../api/jmap-types.ts';
import * as css from './styles.css.ts';

export interface AclEditorProps {
  /** The mailbox whose ACL is edited. */
  mailboxId: Id;
  /** Optional display name for the mailbox (untrusted → bidi-isolated). */
  mailboxName?: string;
  /** The ACL client (injected by E9; a fake in tests). */
  client: AclClient;
}

/** A labelled checkbox for one RFC 4314 right, wired to a change handler. */
function RightCheckbox(props: {
  right: AclRight;
  checked: boolean;
  disabled: boolean;
  domId: string;
  onToggle: (right: AclRight, on: boolean) => void;
}): JSX.Element {
  const descId = `${props.domId}-desc`;
  return (
    <div class={css.rightRow}>
      <input
        type="checkbox"
        class={css.checkbox}
        id={props.domId}
        checked={props.checked}
        disabled={props.disabled}
        aria-describedby={descId}
        data-testid={props.domId}
        onChange={(e) => props.onToggle(props.right, e.currentTarget.checked)}
      />
      <label class={css.rightLabel} for={props.domId}>
        <span class={css.rightName}>{t(`sharing-right-${props.right}-label`)}</span>
        <span class={css.rightDesc} id={descId}>
          {t(`sharing-right-${props.right}-desc`)}
        </span>
      </label>
    </div>
  );
}

export function AclEditor(props: AclEditorProps): JSX.Element {
  onMount(() => void loadCatalog('sharing'));

  const [rights, { refetch }] = createResource<MailboxRights, Id>(
    () => props.mailboxId,
    (mailboxId) => props.client.getMailboxRights(mailboxId),
  );

  const [busy, setBusy] = createSignal(false);
  const [opError, setOpError] = createSignal<string | null>(null);

  const canEdit = createMemo(() => canAdminister(rights()?.myRights ?? ''));

  /** Run a mutation, then refetch; surface a failure without wedging the view. */
  async function run(op: () => Promise<void>): Promise<void> {
    setBusy(true);
    setOpError(null);
    try {
      await op();
      await refetch();
    } catch {
      setOpError(t('sharing-op-failed'));
    } finally {
      setBusy(false);
    }
  }

  function toggleEntryRight(entry: AclEntry, right: AclRight, on: boolean): void {
    void run(() => props.client.grant(props.mailboxId, entry.identifier, toggleRight(entry.rights, right, on)));
  }

  function removeEntry(entry: AclEntry): void {
    void run(() => props.client.revoke(props.mailboxId, entry.identifier));
  }

  // ── add-grant form state ───────────────────────────────────────────────────
  const [newIdentifier, setNewIdentifier] = createSignal('');
  const [newBits, setNewBits] = createSignal<Set<AclRight>>(new Set<AclRight>());

  function toggleNewBit(right: AclRight, on: boolean): void {
    setNewBits((prev) => {
      const next = new Set<AclRight>(prev);
      if (on) next.add(right);
      else next.delete(right);
      return next;
    });
  }

  function submitGrant(e: Event): void {
    e.preventDefault();
    const id = newIdentifier().trim();
    if (id.length === 0) return;
    void run(async () => {
      await props.client.grant(props.mailboxId, id, serializeRights(newBits()));
      setNewIdentifier('');
      setNewBits(new Set<AclRight>());
    });
  }

  const myHeld = createMemo(() => ACL_RIGHTS.filter((r) => hasRight(rights()?.myRights ?? '', r)));

  return (
    <section class={css.wrap} aria-label={t('sharing-editor-label')} data-testid="acl-editor">
      <div>
        <h2 class={css.heading}>
          {props.mailboxName !== undefined
            ? t('sharing-title-named', { mailbox: isolate(props.mailboxName) })
            : t('sharing-title')}
        </h2>
        <p class={css.meta}>{t('sharing-intro')}</p>
      </div>

      <Show when={rights.loading}>
        <p class={css.meta}>{t('sharing-loading')}</p>
      </Show>
      <Show when={rights.error as unknown}>
        <p class={css.error} role="alert">
          {t('sharing-load-failed')}
        </p>
      </Show>

      <Show when={rights()}>
        {(data) => (
          <>
            {/* MYRIGHTS for the current user */}
            <div>
              <p class={css.subHeading}>{t('sharing-your-access')}</p>
              <div class={css.myRightsRow} data-testid="my-rights">
                <Show
                  when={myHeld().length > 0}
                  fallback={<span class={css.meta}>{t('sharing-no-access')}</span>}
                >
                  <For each={myHeld()}>
                    {(r) => <span class={css.chip}>{t(`sharing-right-${r}-label`)}</span>}
                  </For>
                </Show>
              </div>
            </div>

            {/* read-only notice when the user cannot administer */}
            <Show when={!canEdit()}>
              <p class={css.notice} role="note" data-testid="readonly-notice">
                {t('sharing-readonly')}
              </p>
            </Show>

            <Show when={opError()}>
              <p class={css.error} role="alert">
                {opError()}
              </p>
            </Show>

            {/* one fieldset per ACL entry */}
            <div>
              <p class={css.subHeading}>{t('sharing-grants')}</p>
              <Show
                when={data().acl.length > 0}
                fallback={<p class={css.meta}>{t('sharing-no-grants')}</p>}
              >
                <For each={data().acl}>
                  {(entry) => (
                    <fieldset class={css.entryCard} data-testid="acl-entry" data-identifier={entry.identifier}>
                      <div class={css.entryHeader}>
                        <legend class={css.legend}>{isolate(entry.identifier)}</legend>
                        <Show when={canEdit()}>
                          <button
                            type="button"
                            class={css.dangerButton}
                            disabled={busy()}
                            data-testid="remove-grant"
                            onClick={() => removeEntry(entry)}
                          >
                            {t('sharing-remove')}
                          </button>
                        </Show>
                      </div>
                      <div class={css.rightsGrid}>
                        <For each={ACL_RIGHTS}>
                          {(r) => (
                            <RightCheckbox
                              right={r}
                              checked={hasRight(entry.rights, r)}
                              disabled={!canEdit() || busy()}
                              domId={`acl-${entry.identifier}-${r}`}
                              onToggle={(right, on) => toggleEntryRight(entry, right, on)}
                            />
                          )}
                        </For>
                      </div>
                    </fieldset>
                  )}
                </For>
              </Show>
            </div>

            {/* add-grant form — only when the user may administer */}
            <Show when={canEdit()}>
              <form class={css.addForm} onSubmit={submitGrant} data-testid="add-grant-form">
                <p class={css.subHeading}>{t('sharing-add-heading')}</p>
                <div class={css.formRow}>
                  <label class={css.rightName} for="acl-new-identifier">
                    {t('sharing-identifier-label')}
                  </label>
                  <input
                    class={css.input}
                    id="acl-new-identifier"
                    type="text"
                    value={newIdentifier()}
                    placeholder={t('sharing-identifier-placeholder')}
                    data-testid="new-identifier"
                    onInput={(e) => setNewIdentifier(e.currentTarget.value)}
                  />
                </div>
                <fieldset class={css.rightsGrid}>
                  <legend class={css.subHeading}>{t('sharing-rights-legend')}</legend>
                  <For each={ACL_RIGHTS}>
                    {(r) => (
                      <RightCheckbox
                        right={r}
                        checked={newBits().has(r)}
                        disabled={busy()}
                        domId={`acl-new-${r}`}
                        onToggle={toggleNewBit}
                      />
                    )}
                  </For>
                </fieldset>
                <div class={css.formRow}>
                  <button
                    type="submit"
                    class={css.button}
                    disabled={busy() || newIdentifier().trim().length === 0}
                    data-testid="submit-grant"
                  >
                    {t('sharing-add-grant')}
                  </button>
                </div>
              </form>
            </Show>
          </>
        )}
      </Show>
    </section>
  );
}

export default AclEditor;

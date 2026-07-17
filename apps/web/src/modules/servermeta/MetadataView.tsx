// Server / mailbox METADATA view (RFC 5464) — t13 e8, plan §Workstream-2 E8.
//
// Lists the GETMETADATA annotations for a mailbox, or the server-level entries
// when no mailbox is given (RFC 5464 empty-mailbox scope). Editing (SETMETADATA
// write + NIL remove) is GUARDED behind the `canEdit` prop — read-only by default.
// The upstream IMAP server is the real enforcer; the gate is UX honesty. E9 mounts
// this and injects the production `AclClient` + the edit permission.

import { createResource, createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { t, loadCatalog, isolate } from '../../i18n';
import type { AclClient, MetadataEntry } from '../../api/acl-types.ts';
import type { Id } from '../../api/jmap-types.ts';
import * as css from './styles.css.ts';

export interface MetadataViewProps {
  /** The mailbox scope; `null`/omitted = server-level annotations. */
  mailboxId?: Id | null;
  /** Optional display name for the mailbox (untrusted → bidi-isolated). */
  mailboxName?: string;
  /** The ACL/metadata client (injected by E9; a fake in tests). */
  client: AclClient;
  /** Whether the current user may write (SETMETADATA). Default false → read-only. */
  canEdit?: boolean;
}

/** One metadata entry row; when editable, carries its own inline value editor. */
function EntryRow(props: {
  entry: MetadataEntry;
  canEdit: boolean;
  busy: boolean;
  onSave: (entry: string, value: string) => void;
  onRemove: (entry: string) => void;
}): JSX.Element {
  const [draft, setDraft] = createSignal(props.entry.value ?? '');
  const fieldId = `meta-value-${props.entry.entry}`;
  return (
    <li class={css.row} data-testid="metadata-entry" data-entry={props.entry.entry}>
      <div class={css.rowHeader}>
        <span class={css.mono}>{isolate(props.entry.entry)}</span>
        <Show when={props.canEdit}>
          <button
            type="button"
            class={css.dangerButton}
            disabled={props.busy}
            data-testid="remove-entry"
            onClick={() => props.onRemove(props.entry.entry)}
          >
            {t('servermeta-remove')}
          </button>
        </Show>
      </div>

      <Show
        when={props.canEdit}
        fallback={
          <Show
            when={props.entry.value !== null}
            fallback={<span class={css.unset}>{t('servermeta-unset')}</span>}
          >
            <p class={css.valueText}>{props.entry.value}</p>
          </Show>
        }
      >
        <div class={css.editRow}>
          <label class={css.subHeading} for={fieldId}>
            {t('servermeta-value-label')}
          </label>
          <input
            class={css.input}
            id={fieldId}
            type="text"
            value={draft()}
            data-testid="value-input"
            onInput={(e) => setDraft(e.currentTarget.value)}
          />
          <button
            type="button"
            class={css.button}
            disabled={props.busy}
            data-testid="save-entry"
            onClick={() => props.onSave(props.entry.entry, draft())}
          >
            {t('servermeta-save')}
          </button>
        </div>
      </Show>
    </li>
  );
}

export function MetadataView(props: MetadataViewProps): JSX.Element {
  onMount(() => void loadCatalog('servermeta'));

  const scope = (): Id | null => props.mailboxId ?? null;

  const [entries, { refetch }] = createResource<MetadataEntry[], Id | 'server'>(
    () => scope() ?? 'server',
    () => props.client.getServerMetadata(scope()),
  );

  const [busy, setBusy] = createSignal(false);
  const [opError, setOpError] = createSignal<string | null>(null);
  const canEdit = (): boolean => props.canEdit === true;

  async function run(op: () => Promise<void>): Promise<void> {
    setBusy(true);
    setOpError(null);
    try {
      await op();
      await refetch();
    } catch {
      setOpError(t('servermeta-op-failed'));
    } finally {
      setBusy(false);
    }
  }

  // ── add-entry form ──────────────────────────────────────────────────────────
  const [newEntry, setNewEntry] = createSignal('');
  const [newValue, setNewValue] = createSignal('');

  function submitEntry(e: Event): void {
    e.preventDefault();
    const entry = newEntry().trim();
    if (entry.length === 0) return;
    void run(async () => {
      await props.client.setServerMetadata(scope(), entry, newValue());
      setNewEntry('');
      setNewValue('');
    });
  }

  return (
    <section class={css.wrap} aria-label={t('servermeta-view-label')} data-testid="metadata-view">
      <div>
        <h2 class={css.heading}>
          {props.mailboxName !== undefined
            ? t('servermeta-title-named', { mailbox: isolate(props.mailboxName) })
            : t('servermeta-title')}
        </h2>
        <p class={css.meta}>{t('servermeta-intro')}</p>
      </div>

      <Show when={entries.loading}>
        <p class={css.meta}>{t('servermeta-loading')}</p>
      </Show>
      <Show when={entries.error as unknown}>
        <p class={css.error} role="alert">
          {t('servermeta-load-failed')}
        </p>
      </Show>

      <Show when={!canEdit()}>
        <p class={css.notice} role="note" data-testid="readonly-notice">
          {t('servermeta-readonly')}
        </p>
      </Show>

      <Show when={opError()}>
        <p class={css.error} role="alert">
          {opError()}
        </p>
      </Show>

      <Show when={entries()}>
        {(list) => (
          <div>
            <p class={css.subHeading}>{t('servermeta-entries')}</p>
            <Show
              when={list().length > 0}
              fallback={<p class={css.meta}>{t('servermeta-no-entries')}</p>}
            >
              <ul class={css.list}>
                <For each={list()}>
                  {(entry) => (
                    <EntryRow
                      entry={entry}
                      canEdit={canEdit()}
                      busy={busy()}
                      onSave={(name, value) => void run(() => props.client.setServerMetadata(scope(), name, value))}
                      onRemove={(name) => void run(() => props.client.removeServerMetadata(scope(), name))}
                    />
                  )}
                </For>
              </ul>
            </Show>
          </div>
        )}
      </Show>

      <Show when={canEdit()}>
        <form class={css.addForm} onSubmit={submitEntry} data-testid="add-entry-form">
          <p class={css.subHeading}>{t('servermeta-add-heading')}</p>
          <div class={css.editRow}>
            <label class={css.subHeading} for="meta-new-entry">
              {t('servermeta-entry-label')}
            </label>
            <input
              class={css.input}
              id="meta-new-entry"
              type="text"
              value={newEntry()}
              placeholder={t('servermeta-entry-placeholder')}
              data-testid="new-entry"
              onInput={(e) => setNewEntry(e.currentTarget.value)}
            />
          </div>
          <div class={css.editRow}>
            <label class={css.subHeading} for="meta-new-value">
              {t('servermeta-value-label')}
            </label>
            <input
              class={css.input}
              id="meta-new-value"
              type="text"
              value={newValue()}
              placeholder={t('servermeta-value-placeholder')}
              data-testid="new-value"
              onInput={(e) => setNewValue(e.currentTarget.value)}
            />
          </div>
          <div class={css.editRow}>
            <button
              type="submit"
              class={css.button}
              disabled={busy() || newEntry().trim().length === 0}
              data-testid="submit-entry"
            >
              {t('servermeta-add-entry')}
            </button>
          </div>
        </form>
      </Show>
    </section>
  );
}

export default MetadataView;

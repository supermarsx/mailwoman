// Admin › Integrations › Webhooks (t28-e18; worker f34). Server:
// `crates/mw-server/src/webhooks.rs`, client `api/admin/webhooks.ts`.
//
// Lists every webhook (outbound and inbound) and creates and deletes them:
//   • outbound: an account, a URL and the event types; the server checks the URL
//     through the same egress gate deliveries use and refuses a private, loopback
//     or link-local destination with the field named (`422 /url`);
//   • inbound: an account and the actions it may run.
// The new webhook's secret is shown once, with how to verify or sign with it.
// Delete asks first (`ConfirmDialog`). Every write sends the list's tag as
// `If-Match`; a `412` reloads the list and says why.

import { For, Show, createSignal, onMount, type JSX } from 'solid-js';
import {
  createWebhook,
  deleteWebhook,
  listWebhooks,
  type AdminWebhook,
  type NewWebhook,
  type WebhookDirection,
} from '../../api/admin/webhooks.ts';
import { AdminProblemError } from './shell/adminApi.ts';
import { ConfirmDialog } from './shell/ConfirmDialog.tsx';
import { Feedback, describeAdminError } from './shell/Feedback.tsx';
import { FormField } from './shell/FormField.tsx';
import { loadCatalog, t } from '../../i18n';
import { formatDateTime } from '../../i18n/format.ts';
import * as css from './admin.css.ts';
import * as shellCss from './shell/shell.css.ts';

/** Event wire names → label ids. An unknown name is shown as sent. */
const EVENT_LABELS: Record<string, string> = {
  'message.received': 'admin-webhooks-event-message-received',
  'message.flags': 'admin-webhooks-event-message-flags',
  'message.moved': 'admin-webhooks-event-message-moved',
  'message.deleted': 'admin-webhooks-event-message-deleted',
  'submission.sent': 'admin-webhooks-event-submission-sent',
  'submission.failed': 'admin-webhooks-event-submission-failed',
  'pim.changed': 'admin-webhooks-event-pim-changed',
};

const ACTION_LABELS: Record<string, string> = {
  run_rules: 'admin-webhooks-action-run-rules',
  sieve: 'admin-webhooks-action-sieve',
};

function label(map: Record<string, string>, name: string): string {
  const id = map[name];
  return id ? t(id) : name;
}

function when(iso: string | null | undefined): string {
  if (!iso) return '';
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? iso : formatDateTime(d);
}

function nameOf(w: AdminWebhook): string {
  return w.label !== '' ? w.label : w.id;
}

/** Field errors from a 422, keyed by JSON pointer. */
type FieldErrors = Partial<Record<'/accountId' | '/url' | '/events' | '/actions' | '/label' | '/direction', string>>;

export function Webhooks(): JSX.Element {
  const [list, setList] = createSignal<AdminWebhook[]>([]);
  const [eventKinds, setEventKinds] = createSignal<string[]>(Object.keys(EVENT_LABELS));
  const [actionKinds, setActionKinds] = createSignal<string[]>(Object.keys(ACTION_LABELS));
  const [etag, setEtag] = createSignal<string | null>(null);
  const [state, setState] = createSignal<'loading' | 'ready' | 'unavailable' | 'error'>('loading');
  const [message, setMessage] = createSignal<{ kind: 'success' | 'error' | 'info'; text: string } | null>(null);

  // The add form.
  const [direction, setDirection] = createSignal<WebhookDirection>('outbound');
  const [accountId, setAccountId] = createSignal('');
  const [name, setName] = createSignal('');
  const [url, setUrl] = createSignal('');
  const [events, setEvents] = createSignal<string[]>(['message.received']);
  const [actions, setActions] = createSignal<string[]>(['run_rules']);
  const [errors, setErrors] = createSignal<FieldErrors>({});
  const [saving, setSaving] = createSignal(false);

  const [created, setCreated] = createSignal<{ webhook: AdminWebhook; secret: string } | null>(null);
  const [toDelete, setToDelete] = createSignal<AdminWebhook | null>(null);

  async function reload(): Promise<void> {
    try {
      const r = await listWebhooks();
      if (r.kind === 'unavailable') {
        setState('unavailable');
        return;
      }
      setList(r.data.webhooks);
      if (r.data.eventKinds.length > 0) setEventKinds(r.data.eventKinds);
      if (r.data.actionKinds.length > 0) setActionKinds(r.data.actionKinds);
      setEtag(r.etag ?? r.data.etag);
      setState('ready');
    } catch {
      setState('error');
    }
  }

  onMount(() => {
    void loadCatalog('admin-webhooks');
    void reload();
  });

  function toggle(set: (f: (v: string[]) => string[]) => void, value: string, on: boolean): void {
    set((v) => (on ? [...v.filter((x) => x !== value), value] : v.filter((x) => x !== value)));
  }

  /** Checks the server would refuse anyway, before any request. */
  function localErrors(): FieldErrors {
    const e: FieldErrors = {};
    if (accountId().trim() === '') e['/accountId'] = t('admin-webhooks-error-account');
    if (direction() === 'outbound') {
      if (!/^https?:\/\/\S+$/i.test(url().trim())) e['/url'] = t('admin-webhooks-error-url');
      if (events().length === 0) e['/events'] = t('admin-webhooks-error-events');
    } else if (actions().length === 0) {
      e['/actions'] = t('admin-webhooks-error-actions');
    }
    return e;
  }

  function body(): NewWebhook {
    const b: NewWebhook = { direction: direction(), accountId: accountId().trim(), label: name().trim() };
    if (direction() === 'outbound') {
      b.url = url().trim();
      b.events = events();
    } else {
      b.actions = actions();
    }
    return b;
  }

  /** Map a refusal onto the form, or onto the page message. */
  async function onFailure(err: unknown): Promise<void> {
    if (err instanceof AdminProblemError && err.status === 412) {
      await reload();
      setMessage({ kind: 'error', text: t('admin-webhooks-error-stale') });
      return;
    }
    if (err instanceof AdminProblemError && err.status === 422 && err.field) {
      setErrors({ [err.field]: err.detail ?? describeAdminError(err) } as FieldErrors);
      return;
    }
    setMessage({ kind: 'error', text: describeAdminError(err) });
  }

  async function submit(e: Event): Promise<void> {
    e.preventDefault();
    setMessage(null);
    const local = localErrors();
    setErrors(local);
    if (Object.keys(local).length > 0) return;
    setSaving(true);
    try {
      const made = await createWebhook(body(), etag());
      setCreated({ webhook: made.webhook, secret: made.secret });
      setName('');
      setUrl('');
      await reload();
    } catch (err) {
      await onFailure(err);
    } finally {
      setSaving(false);
    }
  }

  async function confirmDelete(): Promise<void> {
    const w = toDelete();
    if (w === null) return;
    try {
      await deleteWebhook(w.id, etag());
    } catch (err) {
      if (err instanceof AdminProblemError && err.status === 412) {
        setToDelete(null);
        await onFailure(err);
        return;
      }
      throw err;
    }
    setToDelete(null);
    await reload();
    setMessage({ kind: 'success', text: t('admin-webhooks-deleted') });
  }

  return (
    <div class={css.card} data-testid="admin-webhooks">
      <h3 class={css.heading}>{t('admin-webhooks-title')}</h3>
      <p class={css.note}>{t('admin-webhooks-intro')}</p>
      <Feedback kind={message()?.kind ?? 'info'} message={message()?.text} testId="admin-webhooks-message" />
      <Show when={state() === 'unavailable'}>
        <p class={css.note}>{t('admin-webhooks-unavailable')}</p>
      </Show>
      <Show when={state() === 'error'}>
        <p class={css.error} role="alert">
          {t('admin-webhooks-load-error')}
        </p>
      </Show>

      <Show when={state() === 'ready'}>
        <Show when={list().length > 0} fallback={<p class={css.note}>{t('admin-webhooks-empty')}</p>}>
          <div class={css.tableWrap}>
            <table class={css.table} data-testid="admin-webhooks-table">
              <thead>
                <tr>
                  <th scope="col">{t('admin-webhooks-col-label')}</th>
                  <th scope="col">{t('admin-webhooks-col-direction')}</th>
                  <th scope="col">{t('admin-webhooks-col-account')}</th>
                  <th scope="col">{t('admin-webhooks-col-target')}</th>
                  <th scope="col">{t('admin-webhooks-col-created')}</th>
                  <th scope="col">
                    <span class={shellCss.srOnly}>{t('admin-webhooks-delete')}</span>
                  </th>
                </tr>
              </thead>
              <tbody>
                <For each={list()}>
                  {(w) => (
                    <tr data-testid={`webhook-row-${w.id}`}>
                      <td dir="auto">{w.label !== '' ? w.label : t('admin-webhooks-no-label')}</td>
                      <td>
                        {w.direction === 'outbound'
                          ? t('admin-webhooks-direction-outbound')
                          : t('admin-webhooks-direction-inbound')}
                      </td>
                      <td dir="auto">{w.accountId}</td>
                      <td>
                        <Show
                          when={w.direction === 'outbound'}
                          fallback={(w.actions ?? []).map((a) => label(ACTION_LABELS, a)).join(', ')}
                        >
                          <div class={css.mono} dir="ltr">
                            {w.url}
                          </div>
                          <div class={css.note}>{(w.events ?? []).map((e) => label(EVENT_LABELS, e)).join(', ')}</div>
                        </Show>
                      </td>
                      <td>
                        <div>{when(w.createdAt)}</div>
                        <div class={css.note} dir="auto">
                          {t('admin-webhooks-created-by', { who: w.createdBy })}
                        </div>
                        <Show when={w.direction === 'inbound'}>
                          <div class={css.note}>
                            {w.lastUsedAt
                              ? t('admin-webhooks-last-used', { when: when(w.lastUsedAt) })
                              : t('admin-webhooks-never-used')}
                          </div>
                        </Show>
                      </td>
                      <td>
                        <button
                          type="button"
                          class="btn btn--ghost"
                          aria-label={t('admin-webhooks-delete-named', { name: nameOf(w) })}
                          onClick={() => {
                            setMessage(null);
                            setToDelete(w);
                          }}
                        >
                          {t('admin-webhooks-delete')}
                        </button>
                      </td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </Show>

        <form class={css.card} onSubmit={(e) => void submit(e)} aria-labelledby="admin-webhooks-add" novalidate>
          <h4 id="admin-webhooks-add" class={css.subHeading}>
            {t('admin-webhooks-add')}
          </h4>
          <fieldset class={css.card} aria-describedby="admin-webhooks-direction-help">
            <legend>{t('admin-webhooks-add-direction')}</legend>
            <label class="field">
              <input
                type="radio"
                name="admin-webhook-direction"
                value="outbound"
                checked={direction() === 'outbound'}
                onChange={() => setDirection('outbound')}
              />
              <span>
                {t('admin-webhooks-direction-outbound')} — {t('admin-webhooks-add-outbound-help')}
              </span>
            </label>
            <label class="field">
              <input
                type="radio"
                name="admin-webhook-direction"
                value="inbound"
                checked={direction() === 'inbound'}
                onChange={() => setDirection('inbound')}
              />
              <span>
                {t('admin-webhooks-direction-inbound')} — {t('admin-webhooks-add-inbound-help')}
              </span>
            </label>
          </fieldset>
          <FormField
            label={t('admin-webhooks-add-account')}
            help={t('admin-webhooks-add-account-help')}
            error={errors()['/accountId'] ?? null}
          >
            {(f) => (
              <input
                id={f.id}
                type="email"
                autocomplete="off"
                value={accountId()}
                aria-describedby={f.describedBy}
                aria-invalid={f.invalid}
                onInput={(e) => setAccountId(e.currentTarget.value)}
              />
            )}
          </FormField>
          <FormField
            label={t('admin-webhooks-add-label')}
            help={t('admin-webhooks-add-label-help')}
            error={errors()['/label'] ?? null}
          >
            {(f) => (
              <input
                id={f.id}
                type="text"
                maxLength={120}
                value={name()}
                aria-describedby={f.describedBy}
                aria-invalid={f.invalid}
                onInput={(e) => setName(e.currentTarget.value)}
              />
            )}
          </FormField>
          <Show
            when={direction() === 'outbound'}
            fallback={
              <fieldset class={css.card} aria-invalid={Boolean(errors()['/actions'])}>
                <legend>{t('admin-webhooks-add-actions')}</legend>
                <For each={actionKinds()}>
                  {(a) => (
                    <label class="field">
                      <input
                        type="checkbox"
                        checked={actions().includes(a)}
                        onChange={(e) => toggle(setActions, a, e.currentTarget.checked)}
                      />
                      <span>{label(ACTION_LABELS, a)}</span>
                    </label>
                  )}
                </For>
                <Show when={errors()['/actions']}>
                  <p class={css.error} role="alert">
                    {errors()['/actions']}
                  </p>
                </Show>
              </fieldset>
            }
          >
            <FormField label={t('admin-webhooks-add-url')} help={t('admin-webhooks-add-url-help')} error={errors()['/url'] ?? null}>
              {(f) => (
                <input
                  id={f.id}
                  type="url"
                  dir="ltr"
                  inputMode="url"
                  autocomplete="off"
                  value={url()}
                  aria-describedby={f.describedBy}
                  aria-invalid={f.invalid}
                  onInput={(e) => setUrl(e.currentTarget.value)}
                />
              )}
            </FormField>
            <fieldset class={css.card} aria-invalid={Boolean(errors()['/events'])}>
              <legend>{t('admin-webhooks-add-events')}</legend>
              <For each={eventKinds()}>
                {(ev) => (
                  <label class="field">
                    <input
                      type="checkbox"
                      checked={events().includes(ev)}
                      onChange={(e) => toggle(setEvents, ev, e.currentTarget.checked)}
                    />
                    <span>{label(EVENT_LABELS, ev)}</span>
                  </label>
                )}
              </For>
              <Show when={errors()['/events']}>
                <p class={css.error} role="alert">
                  {errors()['/events']}
                </p>
              </Show>
            </fieldset>
          </Show>
          <div class={css.confirmActions}>
            <button type="submit" class="btn btn--primary" disabled={saving()} data-testid="admin-webhooks-create">
              {saving() ? t('admin-webhooks-add-saving') : t('admin-webhooks-add-submit')}
            </button>
          </div>
        </form>
      </Show>

      <Show when={created()}>
        {(c) => (
          <section class={css.card} role="region" aria-labelledby="admin-webhooks-secret" data-testid="admin-webhooks-secret">
            <h4 id="admin-webhooks-secret" class={css.subHeading}>
              {t('admin-webhooks-secret-title', { name: nameOf(c().webhook) })}
            </h4>
            <p class={css.note}>{t('admin-webhooks-secret-once')}</p>
            <p class={css.mono} dir="ltr" data-testid="admin-webhooks-secret-value">
              {c().secret}
            </p>
            <p class={css.note}>
              {c().webhook.direction === 'outbound'
                ? t('admin-webhooks-secret-outbound')
                : t('admin-webhooks-secret-inbound', { id: c().webhook.id })}
            </p>
            <div class={css.confirmActions}>
              <button type="button" class="btn btn--ghost" onClick={() => setCreated(null)}>
                {t('admin-webhooks-secret-done')}
              </button>
            </div>
          </section>
        )}
      </Show>

      <ConfirmDialog
        open={toDelete() !== null}
        title={t('admin-webhooks-delete-title', { name: toDelete() ? nameOf(toDelete() as AdminWebhook) : '' })}
        body={t('admin-webhooks-delete-body')}
        confirmLabel={t('admin-webhooks-delete')}
        onConfirm={confirmDelete}
        onCancel={() => setToDelete(null)}
      />
    </div>
  );
}

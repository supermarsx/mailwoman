// Notification rules + quiet hours (t16 e15 — W15). Backed by the `mw-store` 0017
// `notification_rules` row (rule_json + quiet_hours_json + enabled). Rules match a
// substring on sender/mailbox/subject and either surface or mute a notification;
// quiet hours suppress all notifications within a local time window.

import { createEffect, createResource, createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { SettingsService } from './service.ts';
import type { NotificationConfig, NotificationRule } from './types.ts';
import * as css from './styles.css.ts';

export interface NotificationsProps {
  service?: SettingsService;
}

const DEFAULT_CONFIG: NotificationConfig = {
  enabled: true,
  rules: [],
  quietHours: { enabled: false, start: '22:00', end: '07:00' },
};

function newRuleId(): string {
  return `r-${Date.now().toString(36)}-${Math.floor(Math.random() * 1e6).toString(36)}`;
}

export function Notifications(props: NotificationsProps): JSX.Element {
  const service = props.service ?? new SettingsService();
  onMount(() => void loadCatalog('settings'));

  const [loaded] = createResource<NotificationConfig>(() => service.notifications().catch(() => DEFAULT_CONFIG));
  // Working copy; hydrated from the resource once it settles (exactly once).
  const [config, setConfig] = createSignal<NotificationConfig>(DEFAULT_CONFIG);
  const [hydrated, setHydrated] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');
  const [saved, setSaved] = createSignal(false);

  createEffect(() => {
    const fetched = loaded();
    if (fetched !== undefined && !hydrated()) {
      setConfig(fetched);
      setHydrated(true);
    }
  });

  const view = (): NotificationConfig => config();

  function patch(next: Partial<NotificationConfig>): void {
    setConfig({ ...config(), ...next });
    setSaved(false);
  }

  function patchQuiet(next: Partial<NotificationConfig['quietHours']>): void {
    patch({ quietHours: { ...config().quietHours, ...next } });
  }

  function addRule(): void {
    const rule: NotificationRule = { id: newRuleId(), label: '', match: '', action: 'notify' };
    patch({ rules: [...config().rules, rule] });
  }

  function updateRule(id: string, next: Partial<NotificationRule>): void {
    patch({ rules: config().rules.map((r) => (r.id === id ? { ...r, ...next } : r)) });
  }

  function removeRule(id: string): void {
    patch({ rules: config().rules.filter((r) => r.id !== id) });
  }

  async function save(): Promise<void> {
    setError('');
    setBusy(true);
    try {
      await service.saveNotifications(config());
      setSaved(true);
    } catch (e) {
      setError(e instanceof Error ? e.message : t('settings-notif-error'));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section class={css.section} aria-label={t('settings-notif-title')}>
      <h2 class={css.heading}>{t('settings-notif-title')}</h2>
      <p class={css.prose}>{t('settings-notif-intro')}</p>

      <label class={css.check}>
        <input
          class={css.checkbox}
          type="checkbox"
          aria-label={t('settings-notif-enabled-label')}
          checked={view().enabled}
          onChange={(e) => patch({ enabled: e.currentTarget.checked })}
        />
        <span>{t('settings-notif-enabled-label')}</span>
      </label>

      {/* Quiet hours */}
      <div class={css.field}>
        <span class={css.subheading}>{t('settings-notif-quiet-title')}</span>
        <label class={css.check}>
          <input
            class={css.checkbox}
            type="checkbox"
            aria-label={t('settings-notif-quiet-enabled-label')}
            checked={view().quietHours.enabled}
            onChange={(e) => patchQuiet({ enabled: e.currentTarget.checked })}
          />
          <span>{t('settings-notif-quiet-enabled-label')}</span>
        </label>
        <Show when={view().quietHours.enabled}>
          <div class={css.row}>
            <label class={css.field}>
              <span class={css.label}>{t('settings-notif-quiet-start')}</span>
              <input
                class={css.input}
                type="time"
                aria-label={t('settings-notif-quiet-start')}
                value={view().quietHours.start}
                onInput={(e) => patchQuiet({ start: e.currentTarget.value })}
              />
            </label>
            <label class={css.field}>
              <span class={css.label}>{t('settings-notif-quiet-end')}</span>
              <input
                class={css.input}
                type="time"
                aria-label={t('settings-notif-quiet-end')}
                value={view().quietHours.end}
                onInput={(e) => patchQuiet({ end: e.currentTarget.value })}
              />
            </label>
          </div>
        </Show>
      </div>

      {/* Rules */}
      <div class={css.field}>
        <span class={css.subheading}>{t('settings-notif-rules-title')}</span>
        <ul class={css.list} data-testid="notif-rule-list">
          <For each={view().rules}>
            {(rule) => (
              <li class={css.item}>
                <input
                  class={`${css.input} ${css.grow}`}
                  placeholder={t('settings-notif-rule-match-placeholder')}
                  aria-label={t('settings-notif-rule-match-placeholder')}
                  value={rule.match}
                  onInput={(e) => updateRule(rule.id, { match: e.currentTarget.value })}
                />
                <select
                  class={css.select}
                  aria-label={t('settings-notif-rule-action-label')}
                  value={rule.action}
                  onChange={(e) => updateRule(rule.id, { action: e.currentTarget.value as NotificationRule['action'] })}
                >
                  <option value="notify">{t('settings-notif-rule-notify')}</option>
                  <option value="mute">{t('settings-notif-rule-mute')}</option>
                </select>
                <button type="button" class={css.danger} onClick={() => removeRule(rule.id)}>
                  {t('settings-delete')}
                </button>
              </li>
            )}
          </For>
        </ul>
        <div class={css.actions}>
          <button type="button" class={css.ghost} onClick={addRule} data-testid="notif-add-rule">
            {t('settings-notif-rule-add')}
          </button>
        </div>
      </div>

      <div class={css.actions}>
        <button type="button" class={css.button} disabled={busy()} onClick={() => void save()} data-testid="notif-save">
          {t('settings-save')}
        </button>
        <Show when={saved()}>
          <span class={css.success} role="status" data-testid="notif-saved">
            {t('settings-saved')}
          </span>
        </Show>
      </div>

      <Show when={error() !== ''}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>
    </section>
  );
}

export default Notifications;

// Register an engine plugin, in the Plugins list's detail pane (SettingsFormPage,
// `frame: 'none'`).
//
// A first-party component is registered by its id alone, plus hosts when some
// are given (the server refuses any other manifest key for these ids). A
// third-party component also needs its name, version and declared capabilities;
// the first-party-only capability is not offered, since the server refuses it.
// A refusal is shown at the field it concerns (`already-registered` at the id),
// in the server's own words through `refusalText`.

import { createMemo, type JSX } from 'solid-js';
import {
  FIRST_PARTY_PLUGIN_IDS,
  PLUGIN_CAPABILITIES,
  isHighPowerCapability,
  type PluginCapability,
  type PluginsSlice,
  type RegisterInput,
} from '../../../state/slices/plugins.ts';
import { isolate, t } from '../../../i18n';
import { AdminProblemError } from '../shell/adminApi.ts';
import { SettingsFormPage, type SettingsGroup, type SettingsLoad } from '../shell/templates/SettingsFormPage.tsx';
import { ListEditor } from '../shell/ListEditor.tsx';
import { adminDocsHref } from '../shell/docsLink.ts';
import * as css from '../shell/extras.css.ts';
import { refusalText } from './shared.tsx';

const AREA = 'plugins';
/** The `component` value for "a third-party component". */
const THIRD_PARTY = '';
const HOST = /^(?=.{1,253}$)[a-z0-9](?:[a-z0-9.-]{0,251}[a-z0-9])?(?::\d{1,5})?$/;

export interface RegisterPluginProps {
  slice: PluginsSlice;
  labelledBy: string;
  onRegistered: (id: string) => void;
}

/** The `POST /admin/plugins` body from the form values. */
export function registerInput(v: Readonly<Record<string, unknown>>): RegisterInput {
  const hosts = (v.hosts as string[] | undefined) ?? [];
  if (v.component !== THIRD_PARTY) return hosts.length > 0 ? { id: String(v.component), netAllowlist: hosts } : { id: String(v.component) };
  return {
    id: String(v.id ?? '').trim(),
    name: String(v.name ?? '').trim(),
    version: String(v.version ?? '').trim(),
    capabilities: [...((v.capabilities as PluginCapability[] | undefined) ?? [])],
    netAllowlist: hosts,
  };
}

export function RegisterPlugin(props: RegisterPluginProps): JSX.Element {
  const slice = props.slice;
  const unregistered = (): string[] => FIRST_PARTY_PLUGIN_IDS.filter((id) => !slice.plugins().some((p) => p.id === id));
  const third = (v: Readonly<Record<string, unknown>>): boolean => v.component === THIRD_PARTY;
  const docs = (id: string): string => adminDocsHref(AREA, id);

  const groups = createMemo((): SettingsGroup[] => [
    {
      id: 'component',
      heading: t('plugins-admin-register-component'),
      fields: [
        {
          id: 'component',
          label: t('plugins-admin-register-component'),
          help: t('plugins-admin-register-component-help'),
          docsHref: docs('register'),
          kind: 'select',
          options: [...unregistered().map((id) => ({ value: id, label: id })), { value: THIRD_PARTY, label: t('plugins-admin-register-third-party') }],
        },
        {
          id: 'id',
          label: t('plugins-admin-register-id'),
          help: t('plugins-admin-register-id-hint'),
          docsHref: docs('register'),
          kind: 'text',
          required: true,
          maxLength: 64,
          pattern: '[a-z0-9][a-z0-9_-]{0,63}',
          patternHint: t('plugins-admin-register-id-pattern'),
          when: third,
        },
        { id: 'name', label: t('plugins-admin-register-name'), help: t('plugins-admin-register-name-help'), docsHref: docs('register'), kind: 'text', required: true, maxLength: 120, when: third },
        { id: 'version', label: t('plugins-admin-register-version'), help: t('plugins-admin-register-version-help'), docsHref: docs('register'), kind: 'text', required: true, maxLength: 40, when: third },
        {
          id: 'capabilities',
          label: t('plugins-admin-register-capabilities'),
          help: t('plugins-admin-register-capabilities-help'),
          docsHref: docs('grants'),
          kind: 'text',
          when: third,
          render: (ctx) => {
            const chosen = (): PluginCapability[] => (ctx.value as PluginCapability[] | undefined) ?? [];
            return (
              <fieldset class={css.checks} id={ctx.id} aria-describedby={ctx.describedBy}>
                <legend class={css.listEmpty}>{t('plugins-admin-register-capabilities')}</legend>
                {/* The first-party-only capability is not offered: the server refuses it. */}
                {PLUGIN_CAPABILITIES.filter((c) => !isHighPowerCapability(c)).map((cap) => (
                  <div class={css.checkRow}>
                    <input
                      id={`${ctx.id}-${cap}`}
                      type="checkbox"
                      checked={chosen().includes(cap)}
                      onChange={(e) => ctx.setValue(e.currentTarget.checked ? [...chosen(), cap] : chosen().filter((c) => c !== cap))}
                    />
                    <label for={`${ctx.id}-${cap}`} class={css.mono}>
                      {cap}
                    </label>
                  </div>
                ))}
              </fieldset>
            );
          },
        },
        {
          id: 'hosts',
          label: t('plugins-admin-register-hosts'),
          help: t('plugins-admin-register-hosts-help'),
          docsHref: docs('network'),
          kind: 'text',
          render: (ctx) => (
            <ListEditor
              id={ctx.id}
              label={t('plugins-admin-register-hosts')}
              items={(ctx.value as string[] | undefined) ?? []}
              onChange={(list) => ctx.setValue(list)}
              describedBy={ctx.describedBy}
              invalid={ctx.invalid}
              lowerCase
              mono
              emptyText={third(ctx.values) ? t('plugins-admin-register-hosts-none-third-party') : t('plugins-admin-register-hosts-none-first-party')}
              validate={(h) => (HOST.test(h) ? null : t('plugins-admin-register-host-invalid', { host: isolate(h) }))}
            />
          ),
        },
      ],
    },
  ]);

  async function load(): Promise<SettingsLoad> {
    return { values: { component: unregistered()[0] ?? THIRD_PARTY, id: '', name: '', version: '', capabilities: [], hosts: [] } };
  }

  async function save(_changed: Record<string, unknown>, ctx: { values: Readonly<Record<string, unknown>> }): Promise<void> {
    const input = registerInput(ctx.values);
    if (await slice.register(input)) {
      props.onRegistered(input.id);
      return;
    }
    const err = slice.lastError();
    const field = err?.code === 'already-registered' ? (ctx.values.component === THIRD_PARTY ? '/id' : '/component') : '/component';
    throw new AdminProblemError(err?.status ?? 400, { detail: err ? refusalText(err) : t('plugins-admin-load-error'), field }, null);
  }

  return (
    <SettingsFormPage
      title={t('plugins-admin-register-heading')}
      frame="none"
      labelledBy={props.labelledBy}
      groups={groups()}
      load={load}
      save={save}
      saveLabel={t('plugins-admin-register-submit')}
      allowUnchanged
      testId="plugin-register"
    />
  );
}

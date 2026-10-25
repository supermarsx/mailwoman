import { For, Show, onMount, type JSX } from 'solid-js';
import { domainBackends, type DomainDetail } from '../../../api/admin/domains.ts';
import { loadCatalog, t } from '../../../i18n';
import * as css from '../../Settings/styles.css.ts';

export function BackendList(props: { domain: DomainDetail; editable: boolean; open: (id: string) => void }): JSX.Element {
  onMount(() => void loadCatalog('admin-domain-backends'));
  const rows = () => domainBackends(props.domain.upstream);
  const selected = (role: 'mail' | 'submission' | 'rules') => rows().filter(row => row.role === role && row.enabled !== false).sort((a, b) => Number(!!b.preferred) - Number(!!a.preferred) || (a.priority ?? 100) - (b.priority ?? 100) || (a.id < b.id ? -1 : a.id > b.id ? 1 : 0))[0]?.id;
  return <section class={css.section} aria-label={t('backend-list-title')}><h4 class={css.heading}>{t('backend-list-title')}</h4><p class={css.prose}>{t('backend-list-help')}</p>
    <Show when={props.domain.upstream.version !== 2}><p class={css.meta}>{t('backend-list-upgrade')}</p></Show>
    <For each={['mail', 'submission', 'rules'] as const}>{role => <div class={css.stack}><h5 class={css.subheading}>{t(`backend-list-role-${role}`)}</h5><Show when={!selected(role)}><p class={css.meta}>{t('backend-list-none')}</p></Show>
      <For each={rows().filter(row => row.role === role)}>{row => <article class={css.section}><h6 class={css.subheading}>{row.id} · {row.protocol.toUpperCase()}</h6><p class={css.prose}>{row.endpoint ? `${row.endpoint.host}:${row.endpoint.port}` : row.sessionUrl}</p><p class={css.meta}>{t('backend-editor-priority')}: {row.priority ?? 100} · {row.enabled === false ? t('domain-editor-disabled') : selected(role) === row.id ? t('backend-list-selected') : t('domain-editor-enabled')}</p><Show when={props.editable}><div class={css.actions}><button type="button" class={css.button} onClick={() => props.open(row.id)} aria-label={`${t('backend-list-edit')} ${row.id}`}>{t('backend-list-edit')}</button></div></Show></article>}</For>
    </div>}</For><Show when={props.editable}><div class={css.actions}><button type="button" class={css.button} onClick={() => props.open('new')}>{t('backend-list-add')}</button></div></Show>
  </section>;
}

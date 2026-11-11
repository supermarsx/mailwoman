// Admin > Compliance: the hub for data-protection controls. Tabs come from the
// registry in tabs.ts; the Overview shows one status card per tab. The controls
// described are configuration; whether a deployment meets a given law is for the
// operator to decide.

import { For, Match, Show, Switch, createResource, lazy, onMount, type Component, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../../i18n';
import { useShell } from '../shell/context.ts';
import { hrefFor, sectionPath } from '../shell/router.ts';
import { SectionTabs } from '../shell/SectionTabs.tsx';
import { AdminSubtabPanel } from '../shell/AdminSubtabPanel.tsx';
import { Icon } from '../shell/icons.tsx';
import * as css from '../admin.css.ts';
import * as ccss from './compliance.css.ts';
import { COMPLIANCE_TABS, canOpenComplianceTab, type ComplianceTab } from './tabs.ts';

const loaded = new Map<string, Component>();
function tabComponent(tab: ComplianceTab): Component {
  let c = loaded.get(tab.id);
  if (c === undefined) { c = lazy(tab.load) as Component; loaded.set(tab.id, c); }
  return c;
}

function AreaCard(props: { tab: ComplianceTab; open: (path: string) => void }): JSX.Element {
  const [status] = createResource(async () => (props.tab.status ? await props.tab.status().catch(() => undefined) : undefined));
  const path = sectionPath('compliance', [props.tab.id]);
  return (
    <a class={ccss.areaCard} href={hrefFor(path)} data-testid={`compliance-card-${props.tab.id}`}
      onClick={(e) => { if (e.button !== 0 || e.ctrlKey || e.metaKey || e.shiftKey || e.altKey) return; e.preventDefault(); props.open(path); }}>
      <h3><Icon name={props.tab.icon} /> {t(props.tab.label)}</h3>
      <p>{t(props.tab.description)}</p>
      <Show when={status()}>{(s) => <span class={ccss.stateLine} data-state={s().state}>{s().text}</span>}</Show>
    </a>
  );
}

export function Overview(props: { tabs: readonly ComplianceTab[]; open: (path: string) => void }): JSX.Element {
  return (
    <div class={ccss.cards} data-testid="compliance-overview">
      <For each={props.tabs}>{(tab) => <AreaCard tab={tab} open={props.open} />}</For>
    </div>
  );
}

export function AdminCompliance(): JSX.Element {
  const shell = useShell();
  onMount(() => { void loadCatalog('admin-compliance'); void loadCatalog('admin-data-requests'); void loadCatalog('admin-retention'); void loadCatalog('admin-cookies'); void loadCatalog('privacy'); void loadCatalog('admin-erasure'); });
  const tabs = (): ComplianceTab[] => COMPLIANCE_TABS.filter((tab) => canOpenComplianceTab(shell.can, tab));
  const selected = (): string => shell.router.route().rest[0] ?? 'overview';
  const current = (): ComplianceTab | undefined => tabs().find((tab) => tab.id === selected());
  const open = (path: string): void => void shell.router.navigate(path);
  const items = () => [
    { id: 'overview', label: t('admin-crumb-overview'), path: sectionPath('compliance', ['overview']) },
    ...tabs().map((tab) => ({ id: tab.id, label: t(tab.label), path: sectionPath('compliance', [tab.id]) })),
  ];
  return (
    <section class={css.section} data-testid="admin-compliance">
      <h2 class={css.heading}>{t('compliance-title')}</h2>
      <p class={css.note}>{t('compliance-intro')}</p>
      <SectionTabs label={t('compliance-title')} items={items()} selected={selected()} navigate={open} />
      <AdminSubtabPanel>
          <Switch>
            <Match when={selected() === 'overview'}><Overview tabs={tabs()} open={open} /></Match>
            <Match when={current()} keyed>{(tab) => { const C = tabComponent(tab); return <C />; }}</Match>
            <Match when={true}><p role="alert">{t('privacy-tab-denied')}</p></Match>
          </Switch>
      </AdminSubtabPanel>
    </section>
  );
}

export default AdminCompliance;

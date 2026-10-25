import { Icon } from '../../../components/Icon.tsx';
import { For } from 'solid-js';
import { t } from '../../../i18n';
import { usePersonalPreferences } from '../shell/personalPreferences.tsx';
import { dashboardPreferences, type DashboardWidgetId } from './preferences.ts';
import * as css from '../../Settings/styles.css.ts';

export function DashboardPreferencesFields() {
  const prefs = usePersonalPreferences();
  const value = () => dashboardPreferences(prefs.data()['dashboard']);
  function move(id: DashboardWidgetId, direction: number) {
    const current = value(); const order = [...current.order]; const index = order.indexOf(id); const target = index + direction;
    if (target < 0 || target >= order.length) return;
    [order[index], order[target]] = [order[target]!, order[index]!];
    prefs.patch({ dashboard: { ...current, order } });
  }
  return <fieldset id="dashboard-preferences" class={css.section} disabled={prefs.loading() || !!prefs.error()}>
    <legend class={css.heading}>{t('admin-personal-dashboard')}</legend>
    <p class={css.prose}>{t('admin-personal-dashboard-help')}</p>
    <label class={css.field}>{t('admin-personal-dashboard-refresh')}<select class={css.select} value={value().refreshSecs} onChange={e => prefs.patch({ dashboard: { ...value(), refreshSecs: Number(e.currentTarget.value) } })}>
      <option value="0">{t('admin-personal-dashboard-manual')}</option><For each={[30,60,300]}>{seconds => <option value={seconds}>{t('admin-personal-dashboard-seconds', { seconds })}</option>}</For>
    </select></label>
    <p class={css.prose}>{t('admin-personal-dashboard-refresh-help')}</p>
    <For each={value().order}>{(id, index) => <div class={css.actions}>
      <label><input type="checkbox" checked={!value().hidden.includes(id)} onChange={e => prefs.patch({ dashboard: { ...value(), hidden: e.currentTarget.checked ? value().hidden.filter(v => v !== id) : [...value().hidden, id] } })} /> {t(`admin-personal-dashboard-${id}`)}</label>
      <button type="button" class={css.button} aria-label={t('admin-personal-dashboard-move-up', { name: t(`admin-personal-dashboard-${id}`) })} disabled={index() === 0} onClick={() => move(id, -1)}><Icon name="chevronUp" /></button>
      <button type="button" class={css.button} aria-label={t('admin-personal-dashboard-move-down', { name: t(`admin-personal-dashboard-${id}`) })} disabled={index() === value().order.length - 1} onClick={() => move(id, 1)}><Icon name="chevronDown" /></button>
    </div>}</For>
  </fieldset>;
}

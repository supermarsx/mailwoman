import { DashboardReorder, reorderDashboard } from './Reorder.tsx';
import { For } from 'solid-js';
import { t } from '../../../i18n';
import { usePersonalPreferences } from '../shell/personalPreferences.tsx';
import { dashboardPreferences, type DashboardWidgetId } from './preferences.ts';
import * as css from '../../Settings/styles.css.ts';

export function DashboardPreferencesFields() {
  const prefs = usePersonalPreferences();
  const value = () => dashboardPreferences(prefs.data()['dashboard']);
  function move(source:DashboardWidgetId,target:DashboardWidgetId){const current=value();prefs.patch({dashboard:{...current,order:reorderDashboard(current.order,source,target)}});}
  return <fieldset id="dashboard-preferences" class={css.section} disabled={prefs.loading() || !!prefs.error()}>
    <legend class={css.heading}>{t('admin-personal-dashboard')}</legend>
    <p class={css.prose}>{t('admin-personal-dashboard-help')}</p>
    <label class={css.field}>{t('admin-personal-dashboard-refresh')}<select class={css.select} value={value().refreshSecs} onChange={e => prefs.patch({ dashboard: { ...value(), refreshSecs: Number(e.currentTarget.value) } })}>
      <option value="0">{t('admin-personal-dashboard-manual')}</option><For each={[30,60,300]}>{seconds => <option value={seconds}>{t('admin-personal-dashboard-seconds', { seconds })}</option>}</For>
    </select></label>
    <p class={css.prose}>{t('admin-personal-dashboard-refresh-help')}</p>
    <label><input type="checkbox" checked={value().keepCompletedChecklistProminent} onChange={e => prefs.patch({ dashboard: { ...value(), keepCompletedChecklistProminent: e.currentTarget.checked } })} /> {t('admin-personal-dashboard-keep-checklist')}</label>
    <p class={css.prose}>{t('admin-personal-dashboard-keep-checklist-help')}</p>
    <For each={value().order}>{id => <div class={css.actions} data-dashboard-preference data-reorder-id={id}>
      <label><input type="checkbox" checked={!value().hidden.includes(id)} onChange={e => prefs.patch({ dashboard: { ...value(), hidden: e.currentTarget.checked ? value().hidden.filter(v => v !== id) : [...value().hidden, id] } })} /> {t(`admin-personal-dashboard-${id}`)}</label>
      <DashboardReorder id={id} order={value().order} selector="[data-dashboard-preference]" move={move}/>

    </div>}</For>
  </fieldset>;
}

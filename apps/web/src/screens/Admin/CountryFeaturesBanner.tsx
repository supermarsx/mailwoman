// A visible warning when a country-dependent feature is switched on while no
// country database is loaded (those features then do nothing, or follow their
// "when the country cannot be told" setting), plus the database status.
import { createResource, For, Show, useContext, type JSX } from 'solid-js';
import { geoipApi, type GeoipApi } from '../../api/admin/geoip.ts';
import { formatDateTime, t } from '../../i18n';
import { ShellContext } from './shell/context.ts';
import { sectionPath } from './shell/router.ts';
import * as css from '../Settings/styles.css.ts';

export function CountryFeaturesBanner(props: { api?: GeoipApi; showStatus?: boolean }): JSX.Element {
  const shell = useContext(ShellContext);
  const [status] = createResource(async () => {
    try { return (await (props.api ?? geoipApi).read()).data.countryFeatures ?? null; } catch { return null; }
  });
  const link = (): string => `#${sectionPath('security', ['geoip'])}`;
  return (
    <Show when={status()}>
      {(s) => (
        <>
          <Show when={s().warning}>
            <section class={css.section} role="alert" data-testid="country-features-warning">
              <h3 class={css.subheading}>{t('admin-geoip-features-warning-title')}</h3>
              <p class={css.prose}>{t('admin-geoip-features-warning-help')}</p>
              <ul><For each={s().dependentFeatures}>{(id) => <li>{t(`admin-geoip-feature-${id}`)}</li>}</For></ul>
              <p class={css.prose}><a href={shell ? link() : '#/security/geoip'}>{t('admin-geoip-features-install')}</a></p>
            </section>
          </Show>
          <Show when={props.showStatus}>
            <p class={css.meta} data-testid="country-database-status">
              {s().loaded
                ? t('admin-geoip-features-status-loaded', { date: s().buildEpoch ? formatDateTime(s().buildEpoch! * 1000) : '-', age: s().ageDays ?? 0 })
                : t('admin-geoip-features-status-none')}
              <Show when={s().stale}> {t('admin-geoip-features-status-stale')}</Show>
            </p>
          </Show>
        </>
      )}
    </Show>
  );
}

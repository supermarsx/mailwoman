// A banner for the signed-in webmail while processing of the person's data is
// restricted (Art.18). It says what is paused and how to lift it, and points to
// Settings > Privacy & data. Nothing renders when there is no restriction.
import { createResource, Show, type JSX } from 'solid-js';
import { loadCatalog, t } from '../i18n';
import { loadRestriction, type RestrictionStatus } from '../api/privacy.ts';
import { formatDate } from '../i18n/format.ts';
import * as css from './restrictionBanner.css.ts';

export function RestrictionBanner(props: { load?: () => Promise<RestrictionStatus> } = {}): JSX.Element {
  const [status] = createResource(async () => {
    try {
      await loadCatalog('your-data');
      const s = await (props.load ?? loadRestriction)();
      return s.active ? s : null;
    } catch { return null; }
  });
  return <Show when={status()}>{s => (
    <div class={css.banner} role="status" data-testid="restriction-banner">
      <span>{t('yd-banner', { scopes: s().scopes.map(x => t(`yd-scope-${x}`)).join(', ') })}</span>
      <Show when={s().expiresAt}>{e => <span>{t('yd-restriction-until', { date: formatDate(e() * 1000) })}</span>}</Show>
      <span>{t('yd-banner-where')}</span>
    </div>
  )}</Show>;
}

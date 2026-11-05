import { For, Show, createResource, type JSX } from 'solid-js';
import { apiFetch } from '../../api/http.ts';
import { basePath } from '../../api/basePath.ts';
import { loadCatalog, t } from '../../i18n';
import * as css from './styles.css.ts';

interface BreachNotice { id: string; reference: string; subject: string; body: string; acknowledged: boolean }

async function fetchNotices(): Promise<BreachNotice[]> {
  const r = await apiFetch(`${basePath()}/api/privacy/breach-notices`, { credentials: 'same-origin' });
  if (!r.ok) throw new Error(`breach-notices ${r.status}`);
  const body = (await r.json()) as { notices?: BreachNotice[] };
  return Array.isArray(body.notices) ? body.notices : [];
}

async function acknowledge(id: string): Promise<void> {
  await apiFetch(`${basePath()}/api/privacy/breach-notices/ack`, {
    method: 'POST', credentials: 'same-origin', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ id }),
  });
}

/** Notices from the administrator about a personal data breach, shown until acknowledged. Nothing shows while loading, on error or with nothing unread. */
export function BreachNoticeBanner(props: { load?: () => Promise<BreachNotice[]>; ack?: (id: string) => Promise<void> }): JSX.Element {
  void loadCatalog('security');
  const [notices, { refetch }] = createResource(() => (props.load ?? fetchNotices)().catch(() => [] as BreachNotice[]));
  const unread = (): BreachNotice[] => (notices() ?? []).filter((n) => !n.acknowledged);
  return (
    <Show when={unread().length > 0}>
      <div data-testid="breach-notice-banner">
        <For each={unread()}>{(n) => (
          <section class={css.prose} role="status" aria-label={t('breach-notice-heading')} data-testid={`breach-notice-${n.id}`}>
            <strong>{n.subject}</strong>
            <p>{n.body}</p>
            <p>{t('breach-notice-reference', { reference: n.reference })}</p>
            <button type="button" onClick={() => void (props.ack ?? acknowledge)(n.id).then(() => refetch())}>{t('breach-notice-ack')}</button>
          </section>
        )}</For>
      </div>
    </Show>
  );
}
export default BreachNoticeBanner;

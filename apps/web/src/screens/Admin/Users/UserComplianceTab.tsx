import { createResource, For, Show, type JSX } from 'solid-js';
import { t, formatDate } from '../../../i18n/index.ts';
import { AsyncPending } from '../../../components/AsyncState.tsx';
import { listDataRequests } from '../../../api/admin/dataRequests.ts';
import { describeAdminError } from '../shell/Feedback.tsx';
import { hrefFor, sectionPath } from '../shell/router.ts';
import * as css from '../admin.css.ts';
import * as ux from './userDetail.css.ts';

/** The data requests filed for this account's addresses; opened (and fetched) on the tab's first visit. */
export function UserComplianceTab(props: { addresses: readonly string[]; onCount?: (open: number) => void }): JSX.Element {
  // The addresses arrive with the identity; the fetch waits for them.
  const [requests] = createResource(() => (props.addresses.length > 0 ? props.addresses : false), async (addresses) => {
    const wanted = addresses.map((a) => a.toLocaleLowerCase());
    const list = await listDataRequests({ q: addresses[0] ?? '' });
    const mine = list.requests.filter((r) => wanted.includes(r.address.toLocaleLowerCase()));
    props.onCount?.(mine.filter((r) => !['completed', 'rejected', 'cancelled', 'expired', 'downloaded'].includes(r.status)).length);
    return mine;
  });
  return <section class={css.card} aria-label={t('admin-user-tab-compliance')}>
    <h3>{t('admin-user-compliance-title')}</h3>
    <p class={css.note}>{t('admin-user-compliance-help')}</p>
    <Show when={requests.loading}><AsyncPending variant="status" /></Show>
    <Show when={requests.error}><p role="alert">{describeAdminError(requests.error)}</p></Show>
    <Show when={!requests.error && requests.state === 'ready'}>
      <Show when={(requests()?.length ?? 0) > 0} fallback={<p class={css.note}>{t('admin-user-compliance-empty')}</p>}>
        <ul class={ux.list} aria-label={t('admin-user-compliance-title')}>
          <For each={requests()}>{(r) => <li><a href={hrefFor(sectionPath('compliance', ['data-requests']))}>{r.kind}</a> <span class={ux.status}>{r.status}</span> <span class={css.note}>{t('admin-user-compliance-due', { date: formatDate(r.dueAt * 1000) })}</span></li>}</For>
        </ul>
      </Show>
    </Show>
    <a href={hrefFor(sectionPath('compliance', ['data-requests']))}>{t('admin-user-compliance-open-all')}</a>
  </section>;
}

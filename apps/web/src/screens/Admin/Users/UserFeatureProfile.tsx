import { createResource, createSignal, For, onMount, Show, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../../i18n';
import { assignProfile, profileTargetId, profileTargetCount } from '../../../api/admin/usersTable.ts';
import { resourceValue } from '../../../components/resourceValue.ts';
import { toast } from '../../../components/toast/index.ts';
import { featuresApi } from '../Features/api.ts';
import { describeAdminError, Feedback } from '../shell/Feedback.tsx';
import * as css from '../admin.css.ts';

/** Direct assignment belongs to the mailbox identity, regardless of its backend. */
export function UserFeatureProfile(props: { accountId: string }): JSX.Element {
  onMount(() => void loadCatalog('admin-users'));
  const [catalogue, { refetch }] = createResource(() => props.accountId, () => featuresApi.load());
  const data = () => resourceValue(catalogue)?.data;
  const target = () => data() ? profileTargetId(data()!, props.accountId) : null;
  const own = () => data()?.assignments.filter(a => a.scopeKind === 'user' && a.scopeId === target())
    .sort((a, b) => b.priority - a.priority || a.profileId.localeCompare(b.profileId))[0];
  const [choice, setChoice] = createSignal<string>();
  const selected = () => choice() ?? own()?.profileId ?? '';
  const [busy, setBusy] = createSignal(false);
  async function save(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (busy() || !data()) return;
    setBusy(true);
    try {
      await assignProfile(props.accountId, selected() || null, own()?.profileId ?? null);
      await refetch();
      setChoice(undefined);
      toast.success(t('admin-saved'));
    } catch (error) { toast.error(describeAdminError(error)); }
    finally { setBusy(false); }
  }
  return <section class={css.card} aria-label={t('admin-userlist-col-profile')}>
    <h3>{t('admin-userlist-col-profile')}</h3>
    <Feedback kind="error" message={catalogue.error ? describeAdminError(catalogue.error) : null} />
    <form onSubmit={event => void save(event)} aria-label={t('admin-userlist-bulk-profile')}>
      <label class="field"><span>{t('admin-userlist-col-profile')}</span>
        <select value={selected()} disabled={busy() || !data() || target() === null} onChange={event => setChoice(event.currentTarget.value)}>
          <option value="">{t('admin-userlist-profile-default')}</option>
          <For each={data()?.profiles.map(profile => profile.id)}>{id => <option value={id}>{data()?.profiles.find(profile => profile.id === id)?.name}</option>}</For>
        </select>
      </label>
      <p class={css.note}>{t('admin-users-profile-help')}</p>
      <Show when={data() && target() === null}><p class={css.note}>{t(profileTargetCount(data()!, props.accountId) > 1 ? 'admin-users-profile-ambiguous' : 'admin-users-profile-unavailable')}</p></Show>
      <button class="btn" disabled={busy() || !data() || target() === null || selected() === (own()?.profileId ?? '')}>{t('admin-userlist-confirm-profile-action')}</button>
      <Show when={catalogue.error}><button type="button" class="btn btn--ghost" onClick={() => void refetch()}>{t('common-retry')}</button></Show>
    </form>
  </section>;
}

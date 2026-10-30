// Admin → Feature profiles (t39 N9): named sets of feature switches, assigned to
// domains, groups and users.
//
// The list is a `ListDetailPage` of the profiles at `#/features`; a profile's
// detail at `#/features/<id>` edits its name and its switches (`ProfileEditor`)
// and lists where it applies (`Assignments`). `#/features/+new` creates one.
//
// The server holds the whole catalogue as one document with one revision. Every
// write sends the revision read last (`If-Match`); when another administrator
// changed the catalogue first the server refuses (412) and the screen reloads it
// and says so. Profiles only change native accounts: a proxy sign-in follows the
// upstream server's own policy (the server keeps those apart).

import { Show, createMemo, createSignal, onMount, useContext, type JSX } from 'solid-js';
import { isolate, loadCatalog, t } from '../../../i18n';
import { ShellContext } from '../shell/context.ts';
import { ListDetailPage } from '../shell/templates/ListDetailPage.tsx';
import { ConfirmDialog } from '../shell/ConfirmDialog.tsx';
import { Feedback, describeAdminError } from '../shell/Feedback.tsx';
import { AdminProblemError } from '../shell/adminApi.ts';
import { sectionPath, type AdminRoute } from '../shell/router.ts';
import type { Column } from '../shell/DataTable.tsx';
import { AsyncPending } from '../../../components/AsyncState.tsx';
import { featuresApi, profileIdFor, type FeatureCatalogue, type FeatureProfile, type FeaturesApi, type ProfileAssignment, type ProfileValue } from './api.ts';
import { ProfileEditor } from './ProfileEditor.tsx';
import { Assignments } from './Assignments.tsx';
import { Preview } from './Preview.tsx';
import type { Values } from './model.ts';
import * as css from '../shell/extras.css.ts';

const SECTION = 'features';
/** The new-profile form's place in the list (`+` is not an id character). */
export const NEW = '+new';

export interface AdminFeaturesProps {
  api?: FeaturesApi;
  route?: AdminRoute;
  navigate?: (path: string) => void;
}

export function AdminFeatures(props: AdminFeaturesProps): JSX.Element {
  const api = props.api ?? featuresApi;
  const shell = useContext(ShellContext);
  const route = (): AdminRoute => props.route ?? shell?.router.route() ?? { section: SECTION, rest: [] };
  const navigate = (path: string): void => {
    if (props.navigate) props.navigate(path);
    else void shell?.router.navigate(path);
  };
  const [catalogue, setCatalogue] = createSignal<FeatureCatalogue>();
  const [etag, setEtag] = createSignal<string | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [loadFailed, setLoadFailed] = createSignal(false);
  const [error, setError] = createSignal<string>();
  const [notice, setNotice] = createSignal<string>();
  const [query, setQuery] = createSignal('');
  const [deleting, setDeleting] = createSignal<FeatureProfile>();
  /** t98: the starting values of the new-profile form (Clone). */
  const [draft, setDraft] = createSignal<{ name: string; values: Values }>();

  async function load(): Promise<void> {
    setLoading(true);
    try {
      const r = await api.load();
      setCatalogue(r.data);
      setEtag(r.etag);
      setLoadFailed(false);
    } catch (e) {
      setLoadFailed(true);
      setError(describeAdminError(e));
    } finally {
      setLoading(false);
    }
  }
  onMount(() => {
    void loadCatalog('admin-features');
    void loadCatalog('admin-settings');
    void load();
  });

  /** Run one write with the current revision; on a stale revision reload and say so. */
  async function write(fn: (tag: string) => Promise<string | null>, done: string): Promise<boolean> {
    setError(undefined);
    setNotice(undefined);
    const tag = etag();
    if (!tag) {
      setError(t('admin-features-not-loaded'));
      return false;
    }
    try {
      const next = await fn(tag);
      if (next) setEtag(next);
      await load();
      setNotice(done);
      return true;
    } catch (e) {
      if (e instanceof AdminProblemError && e.status === 412) {
        await load();
        setError(t('admin-features-stale'));
      } else {
        setError(describeAdminError(e));
      }
      return false;
    }
  }

  const profiles = (): FeatureProfile[] => catalogue()?.profiles ?? [];
  const find = (id: string): FeatureProfile | undefined => profiles().find((p) => p.id === id);
  const assignmentsOf = (id: string): ProfileAssignment[] => (catalogue()?.assignments ?? []).filter((a) => a.profileId === id);
  const shown = createMemo(() => {
    const q = query().trim().toLowerCase();
    return profiles().filter((p) => q === '' || p.name.toLowerCase().includes(q) || p.id.includes(q));
  });

  const columns: Column<FeatureProfile>[] = [
    { id: 'name', header: t('admin-features-col-name'), cell: (p) => <span dir="auto">{p.name}</span> },
    { id: 'values', header: t('admin-features-col-values'), cell: (p) => <>{t('admin-features-values-count', { count: Object.keys(p.values).length })}</> },
    { id: 'assigned', header: t('admin-features-col-assigned'), cell: (p) => <>{t('admin-features-assigned-count', { count: assignmentsOf(p.id).length })}</> },
  ];

  async function save(profile: FeatureProfile | undefined, name: string, values: Record<string, ProfileValue>): Promise<boolean> {
    const id = profile?.id ?? profileIdFor(name, profiles().map((p) => p.id));
    const ok = await write((tag) => api.putProfile(id, { name, values }, tag), t('admin-features-saved', { name: isolate(name) }));
    if (ok && !profile) setTimeout(() => navigate(sectionPath(SECTION, [id])), 0);
    return ok;
  }

  // t98 (t75 pattern): the detail stays mounted while the catalogue reloads after a
  // write; only a first load shows the pending state.
  const detail = (id: string): JSX.Element => (
    <Show when={catalogue()} fallback={loading() ? <AsyncPending /> : <Feedback kind="error" message={error()} />}>
      {(current) => detailOf(id, current)}
    </Show>
  );
  const detailOf = (id: string, current: () => FeatureCatalogue): JSX.Element => {
    if (id === NEW) {
      return (
        <Show when={draft() ?? true} keyed>
          {(_draft) => <ProfileEditor catalogue={current()} {...(draft() ? { initial: draft()! } : {})} importDocument={api.importDocument?.bind(api)}
            onSave={async (name, values) => { const ok = await save(undefined, name, values); if (ok) setDraft(undefined); return ok; }} />}
        </Show>
      );
    }
    return (
      <Show when={find(id)?.id} keyed fallback={loading() ? <AsyncPending /> : <Feedback kind="error" message={t('admin-features-not-found', { id: isolate(id) })} />}>
        {(_id) => {
          const p = (): FeatureProfile => find(id)!;
          return (
          <>
            <ProfileEditor catalogue={current()} profile={p()} onSave={(name, values) => save(p(), name, values)} onDelete={() => setDeleting(p())}
              importDocument={api.importDocument?.bind(api)}
              onClone={(name, values) => { setDraft({ name, values }); navigate(sectionPath(SECTION, [NEW])); }} />
            <Assignments
              catalogue={current()}
              profile={p()}
              onAssign={(a) => write((tag) => api.assign(a, tag), t('admin-features-assigned', { name: isolate(p().name) }))}
              onUnassign={(a) => write((tag) => api.unassign(a, tag), t('admin-features-unassigned', { name: isolate(p().name) }))}
            />
            <Show when={api.effective}>{(load) => <Preview catalogue={current()} load={(subject) => load()(subject)} />}</Show>
          </>
          );
        }}
      </Show>
    );
  };

  return (
    <div class={css.listEditor} data-screen="admin-features">
      <Feedback kind="error" message={error()} testId="features-error" />
      <Feedback kind="success" message={notice()} testId="features-notice" />
      <ListDetailPage<FeatureProfile>
        title={t('admin-features-title')}
        description={t('admin-features-intro')}
        section={SECTION}
        route={route()}
        navigate={navigate}
        actions={<button type="button" class="btn btn--primary" onClick={() => { setDraft(undefined); navigate(sectionPath(SECTION, [NEW])); }}>{t('admin-features-new')}</button>}
        table={{
          caption: t('admin-features-title'),
          columns,
          rows: shown(),
          rowKey: (p) => p.id,
          state: loadFailed() && !catalogue() ? 'error' : loading() && !catalogue() ? 'loading' : 'ready',
          errorText: t('admin-features-load-error'),
          onRetry: () => void load(),
          emptyText: profiles().length === 0 ? t('admin-features-empty') : t('admin-features-no-match'),
          ...(profiles().length === 0 ? { emptyAction: { label: t('admin-features-new'), onClick: () => navigate(sectionPath(SECTION, [NEW])) } } : {}),
          search: { label: t('admin-features-search'), value: query(), onSearch: setQuery, debounceMs: 0 },
        }}
        detailTitle={(id) => (id === NEW ? t('admin-features-new') : (find(id)?.name ?? id))}
        renderDetail={detail}
        testId="admin-features"
      />
      <ConfirmDialog
        open={deleting() !== undefined}
        title={t('admin-features-delete-title', { name: isolate(deleting()?.name ?? '') })}
        body={t('admin-features-delete-body', { count: assignmentsOf(deleting()?.id ?? '').length })}
        confirmLabel={t('admin-features-delete')}
        onConfirm={async () => {
          const p = deleting();
          if (!p) return;
          const ok = await write((tag) => api.deleteProfile(p.id, tag), t('admin-features-deleted', { name: isolate(p.name) }));
          setDeleting(undefined);
          if (ok) navigate(sectionPath(SECTION));
        }}
        onCancel={() => setDeleting(undefined)}
      />
    </div>
  );
}

export default AdminFeatures;

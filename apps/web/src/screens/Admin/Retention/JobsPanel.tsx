// Admin > Retention: the three retention jobs the server schedules itself
// (people's retention rules, the archive check and upload cleanup). Each has an
// on/off switch, its next run, what its last preview would have changed and,
// while a new task is still previewing, the date it starts deleting.

import { For, Show, createResource, createSignal, type JSX } from 'solid-js';
import { formatDateTime, formatNumber, t } from '../../../i18n';
import { adminRequest } from '../shell/adminApi.ts';
import { AdminSubsection } from '../AdminSubsection.tsx';
import { describeAdminError } from '../shell/Feedback.tsx';
import * as css from '../admin.css.ts';

/** One job in `GET /admin/retention/jobs`. */
export interface RetentionJob {
  kind: 'user_retention' | 'retention_archive' | 'gc_uploads';
  taskId: string | null;
  enabled: boolean;
  nextRunAt: number | null;
  /** Until this time a scheduled run only previews; null = deletes. */
  armAfter: number | null;
  previewOnly: boolean;
  olderThanHours?: number | null;
  lastRun: { at: number; status: string; summary: string | null; dryRun: boolean } | null;
  lastPreview: { at: number; wouldChange: number | null; summary: string | null } | null;
}

export interface JobsApi {
  list: () => Promise<{ jobs: RetentionJob[]; now: number }>;
  setEnabled: (kind: RetentionJob['kind'], enabled: boolean) => Promise<RetentionJob>;
}

export const liveJobsApi: JobsApi = {
  list: async () => (await adminRequest<{ jobs: RetentionJob[]; now: number }>('/retention/jobs')).data,
  setEnabled: async (kind, enabled) =>
    (await adminRequest<RetentionJob>(`/retention/jobs/${kind}`, { method: 'PUT', body: { enabled } })).data,
};

const when = (s: number | null | undefined): string => (s ? formatDateTime(s * 1000) : t('admin-retention-never'));

export function JobsPanel(props: { api: JobsApi }): JSX.Element {
  const [data, { mutate }] = createResource(() => props.api.list());
  const [busy, setBusy] = createSignal<string | null>(null);
  const [failure, setFailure] = createSignal<unknown>(null);

  const toggle = async (job: RetentionJob): Promise<void> => {
    setBusy(job.kind);
    setFailure(null);
    try {
      const next = await props.api.setEnabled(job.kind, !job.enabled);
      const cur = data.latest;
      if (cur) mutate({ ...cur, jobs: cur.jobs.map((j) => (j.kind === next.kind ? next : j)) });
    } catch (error) {
      setFailure(error);
    } finally {
      setBusy(null);
    }
  };

  return (
    <AdminSubsection title={t('admin-retention-jobs')} description={t('admin-retention-jobs-intro')}>
      <Show when={data.error}><p role="alert">{describeAdminError(data.error)}</p></Show>
      <Show when={failure()}><p role="alert">{describeAdminError(failure())}</p></Show>
      <Show when={data.error ? undefined : data.latest}>
        {(d) => (
          <table class={css.table} data-testid="retention-jobs">
            <thead>
              <tr>
                <th scope="col">{t('admin-retention-jobs-col-job')}</th>
                <th scope="col">{t('admin-retention-jobs-col-state')}</th>
                <th scope="col">{t('admin-retention-jobs-col-next')}</th>
                <th scope="col">{t('admin-retention-jobs-col-preview')}</th>
              </tr>
            </thead>
            <tbody>
              <For each={d().jobs}>{(job) => (
                <tr data-testid={`retention-job-${job.kind}`}>
                  <th scope="row">
                    {t(`admin-retention-job-${job.kind}`)}
                    <div class={css.note}>{t(`admin-retention-job-${job.kind}-what`)}</div>
                  </th>
                  <td>
                    <label>
                      <input
                        type="checkbox"
                        role="switch"
                        checked={job.enabled}
                        disabled={busy() !== null}
                        onChange={() => void toggle(job)}
                        data-testid={`retention-job-switch-${job.kind}`}
                      />
                      {' '}{job.enabled ? t('admin-retention-state-on') : t('admin-retention-state-off')}
                    </label>
                    <Show when={job.enabled && job.previewOnly && job.armAfter}>
                      {(at) => (
                        <div role="note" data-testid={`retention-job-arms-${job.kind}`}>
                          {t('admin-retention-jobs-starts', { date: formatDateTime(at() * 1000) })}
                        </div>
                      )}
                    </Show>
                  </td>
                  <td>{job.enabled ? when(job.nextRunAt) : t('admin-retention-none')}</td>
                  <td data-testid={`retention-job-preview-${job.kind}`}>
                    <Show when={job.lastPreview} fallback={t('admin-retention-jobs-no-preview')}>
                      {(p) => t('admin-retention-jobs-would', {
                        count: formatNumber(p().wouldChange ?? 0),
                        date: when(p().at),
                      })}
                    </Show>
                  </td>
                </tr>
              )}</For>
            </tbody>
          </table>
        )}
      </Show>
    </AdminSubsection>
  );
}

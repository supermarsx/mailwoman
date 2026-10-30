// t105: the Diagnostics screen's state: saved runs, the run in progress, and
// the sequenced "Run all".
//
// One store per mounted screen, shared by every tab, so switching tabs keeps the
// results and a run in progress. A suite already running is never started again
// from this page: a second click joins the run in flight (the server's own job
// dedupe and rate limits, t104, cover other tabs and sessions). Failures are
// corner toasts with a Retry action; nothing replaces the page while a check runs.

import { createSignal, type Accessor } from 'solid-js';
import { t } from '../../../i18n';
import { toast } from '../../../components/toast/index.ts';
import { describeAdminError } from '../shell/Feedback.tsx';
import { clearAdminError, toastAdminError } from '../shell/requestErrorToast.ts';
import type { Catalogue, DiagnosticsApi, Job, Run } from './api.ts';

export type Suite = 'server' | 'backends';
export interface RunAllProgress { step: number; steps: number; suite: Suite; completed?: number; total?: number }

export interface DiagnosticsStore {
  runs: Accessor<Run[]>;
  cursor: Accessor<string | null>;
  loading: Accessor<boolean>;
  loaded: Accessor<boolean>;
  unavailable: Accessor<boolean>;
  catalogue: Accessor<Catalogue | undefined>;
  /** The suite being run now, if any. */
  running: Accessor<ReadonlySet<Suite>>;
  /** The card whose Run button started the current run (it shows "Running"). */
  trigger: Accessor<string | undefined>;
  job: Accessor<Job | undefined>;
  progress: Accessor<RunAllProgress | undefined>;
  notice: Accessor<string | undefined>;
  canRunBackends: () => boolean;
  reload: () => Promise<void>;
  more: () => Promise<void>;
  runSuite: (suite: Suite, trigger?: string) => Promise<Run | undefined>;
  runAll: () => Promise<void>;
  cancel: () => Promise<void>;
  dispose: () => void;
}


export function createDiagnosticsStore(api: () => DiagnosticsApi): DiagnosticsStore {
  const [runs, setRuns] = createSignal<Run[]>([]);
  const [cursor, setCursor] = createSignal<string | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [loaded, setLoaded] = createSignal(false);
  const [unavailable, setUnavailable] = createSignal(false);
  const [catalogue, setCatalogue] = createSignal<Catalogue>();
  const [running, setRunning] = createSignal<ReadonlySet<Suite>>(new Set());
  const [trigger, setTrigger] = createSignal<string>();
  const [job, setJob] = createSignal<Job>();
  const [progress, setProgress] = createSignal<RunAllProgress>();
  const [notice, setNotice] = createSignal<string>();
  const inflight = new Map<Suite, Promise<Run | undefined>>();
  let disposed = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let wake: (() => void) | undefined;

  // t104: a failed request is the corner toast with Retry (one per action key);
  // the next success from the same action closes it. Inline only without a ToastRegion.
  const fail = (error: unknown, key: string, retry: () => void | Promise<void>): void => {
    if (disposed) return;
    if (!toastAdminError(error, { key, retry })) setNotice(describeAdminError(error));
  };
  const mark = (suite: Suite, on: boolean): void => {
    const next = new Set(running());
    if (on) next.add(suite); else next.delete(suite);
    setRunning(next);
  };
  const keep = (run: Run): void => { setRuns([run, ...runs().filter((r) => r.id !== run.id)]); };

  async function reload(): Promise<void> {
    setLoading(true);
    try {
      const result = await api().history();
      if (disposed) return;
      if (result.kind === 'unavailable') { setUnavailable(true); return; }
      // A run this page holds that the first page does not (just run, unsaved) stays.
      const fresh = result.data.runs;
      setRuns([...runs().filter((r) => !fresh.some((f) => f.id === r.id) && r.startedAt >= (fresh[0]?.startedAt ?? 0)), ...fresh]);
      setCursor(result.data.nextCursor);
      clearAdminError('diag-history');
    } catch (e) { fail(e, 'diag-history', reload); }
    finally { if (!disposed) { setLoading(false); setLoaded(true); } }
  }
  async function more(): Promise<void> {
    const next = cursor();
    if (!next || loading()) return;
    setLoading(true);
    try {
      const result = await api().history(next);
      if (disposed || result.kind === 'unavailable') return;
      setRuns([...runs(), ...result.data.runs.filter((r) => !runs().some((x) => x.id === r.id))]);
      setCursor(result.data.nextCursor);
      clearAdminError('diag-history');
    } catch (e) { fail(e, 'diag-history', more); }
    finally { if (!disposed) setLoading(false); }
  }
  async function loadCatalogue(): Promise<void> {
    try { const value = await api().catalogue?.(); if (!disposed) setCatalogue(value); } catch { /* the catalogue only explains limits */ }
  }

  async function serverSuite(): Promise<Run | undefined> {
    const result = await api().run();
    if (disposed) return result.run;
    keep(result.run);
    if (!result.historySaved || !result.auditSaved) setNotice(t('admin-diag-not-saved'));
    else if (result.retentionApplied === false) setNotice(t('admin-diag-retention-failed'));
    return result.run;
  }
  function pause(ms: number): Promise<void> {
    return new Promise((resolve) => { wake = resolve; timer = setTimeout(resolve, ms); });
  }
  async function backendSuite(onProgress?: (j: Job) => void): Promise<Run | undefined> {
    const start = api().start;
    if (!start) return undefined;
    let value = await start();
    for (;;) {
      if (disposed) return undefined;
      setJob(value);
      onProgress?.(value);
      if (value.run.status !== 'running') break;
      await pause(1000);
      if (disposed) return undefined;
      value = await api().job!(value.run.id);
    }
    setJob(undefined);
    keep(value.run);
    if(value.termination==='cancelled')toast.info(t('admin-diag-cancelled'),{id:'admin-diag-cancel'});
    setNotice(value.limited ? t('admin-diag-limited') : !value.historySaved ? t('admin-diag-not-saved') : undefined);
    return value.run;
  }

  /** Run one suite; a second call while it runs joins the same run. */
  function runSuite(suite: Suite, from?: string, onProgress?: (j: Job) => void): Promise<Run | undefined> {
    const current = inflight.get(suite);
    if (current) return current;
    setNotice(undefined);
    setTrigger(from);
    mark(suite, true);
    const work = (suite === 'server' ? serverSuite() : backendSuite(onProgress))
      .then((run) => { clearAdminError(`diag-${suite}`); return run; })
      .catch((e: unknown) => { fail(e, `diag-${suite}`, () => runSuite(suite, from).then(() => undefined)); return undefined; })
      .finally(() => { inflight.delete(suite); if (!disposed) { mark(suite, false); if (trigger() === from) setTrigger(undefined); } });
    inflight.set(suite, work);
    return work;
  }

  const canRunBackends = (): boolean => !!api().start && (!api().catalogue || catalogue()?.canRunBackends === true);

  async function runAll(): Promise<void> {
    if (progress()) return;
    const suites: Suite[] = canRunBackends() ? ['server', 'backends'] : ['server'];
    try {
      for (const [index, suite] of suites.entries()) {
        if (disposed) return;
        setProgress({ step: index + 1, steps: suites.length, suite });
        const run = await runSuite(suite, 'all', (j) => setProgress({ step: index + 1, steps: suites.length, suite, completed: j.completed, total: j.total }));
        // A failed request stops the sequence; its toast offers Retry.
        if (!run) return;
      }
      if (!disposed) toast.success(t('admin-diagx-run-all-done'), { id: 'admin-diagnostics-run-all' });
    } finally { if (!disposed) setProgress(undefined); }
  }

  async function cancel(): Promise<void> {
    const current = job();
    if (!current) return;
    try { await api().cancel?.(current.run.id); clearAdminError('diag-cancel'); toast.info(t('admin-diag-cancelling'),{id:'admin-diag-cancel'}); }
    catch (e) { fail(e, 'diag-cancel', cancel); }
  }

  void reload();
  void loadCatalogue();

  return {
    runs, cursor, loading, loaded, unavailable, catalogue, running, trigger, job, progress, notice, canRunBackends,
    reload, more, runSuite: (suite, from) => runSuite(suite, from), runAll, cancel,
    dispose: () => { disposed = true; clearTimeout(timer); wake?.(); },
  };
}

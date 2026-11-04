// "Test a file": the server's own evaluator (attachment_policy.rs) decides for a
// file name, MIME type and size under the stored global policy. Read-only: the
// endpoint (POST /admin/attachments/simulate, settings.read) changes nothing.

import { For, Show, createSignal, type JSX } from 'solid-js';
import { formatBytes, t } from '../../../i18n';
import { adminRequest } from '../shell/adminApi.ts';
import { describeAdminError } from '../shell/Feedback.tsx';
import { AdminSubsection } from '../AdminSubsection.tsx';
import { BYTE_UNITS, toBytes, type ByteUnit } from './rules.ts';
import * as css from './attachments.css.ts';

export interface SimulationResult {
  decision: { downloadAllowed: boolean; previewAllowed: boolean; warning: boolean; reason: string | null };
  matched: string[];
  policy?: unknown;
}
export interface SimulateRequest { filename: string; mime: string; size: number }
export const simulateAttachment = async (body: SimulateRequest): Promise<SimulationResult> =>
  (await adminRequest<SimulationResult>('/attachments/simulate', { method: 'POST', body })).data;

const REASONS = ['downloadsDisabled', 'attachmentTooLarge', 'attachmentBlocked', 'attachmentNotAllowed', 'attachmentContainerRestricted'];

export function AttachmentSimulator(props: { simulate?: typeof simulateAttachment }): JSX.Element {
  const run = props.simulate ?? simulateAttachment;
  const [filename, setFilename] = createSignal('');
  const [mime, setMime] = createSignal('application/octet-stream');
  const [amount, setAmount] = createSignal('0');
  const [unit, setUnit] = createSignal<ByteUnit>('KiB');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [result, setResult] = createSignal<(SimulationResult & { size: number }) | null>(null);

  async function submit(e: Event): Promise<void> {
    e.preventDefault();
    const n = Number(amount().replace(',', '.'));
    if (filename().trim() === '' || !Number.isFinite(n) || n < 0) {
      setError(t('admin-attachments-sim-invalid'));
      return;
    }
    const size = toBytes(n, unit());
    setBusy(true);
    setError(null);
    try {
      setResult({ ...(await run({ filename: filename().trim(), mime: mime().trim(), size })), size });
    } catch (err) {
      setResult(null);
      setError(describeAdminError(err));
    } finally {
      setBusy(false);
    }
  }
  const verdict = (r: SimulationResult): 'blocked' | 'warn' | 'allowed' => (!r.decision.downloadAllowed ? 'blocked' : r.decision.warning ? 'warn' : 'allowed');

  return (
    <AdminSubsection title={t('admin-attachments-sim-title')} description={t('admin-attachments-sim-help')}>
      <form class={css.simulator} onSubmit={(e) => void submit(e)} noValidate>
        <label>{t('admin-attachments-sim-filename')}
          <input type="text" value={filename()} autocapitalize="none" spellcheck={false} onInput={(e) => setFilename(e.currentTarget.value)} />
        </label>
        <label>{t('admin-attachments-sim-mime')}
          <input type="text" value={mime()} autocapitalize="none" spellcheck={false} onInput={(e) => setMime(e.currentTarget.value)} />
        </label>
        <label>{t('admin-attachments-sim-size')}
          <input type="text" inputmode="decimal" value={amount()} onInput={(e) => setAmount(e.currentTarget.value)} />
        </label>
        <label>{t('admin-attachments-bytes-unit')}
          <select value={unit()} onChange={(e) => setUnit(e.currentTarget.value as ByteUnit)}>
            <For each={BYTE_UNITS}>{(u) => <option value={u} selected={u === unit()}>{u}</option>}</For>
          </select>
        </label>
        <button type="submit" class="btn btn--primary" aria-busy={busy()} disabled={busy()}>{t('admin-attachments-sim-run')}</button>
      </form>
      <Show when={error()}>{(message) => <p role="alert">{message()}</p>}</Show>
      <div aria-live="polite">
        <Show when={result()}>{(r) => (
          <div class={css.result} data-testid="simulator-result" data-verdict={verdict(r())}>
            <strong>{t(`admin-attachments-sim-${verdict(r())}`)}</strong>
            <Show when={r().decision.reason}>{(reason) => <span>{REASONS.includes(reason()) ? t(`admin-attachments-reason-${reason()}`) : reason()}</span>}</Show>
            <Show when={r().decision.downloadAllowed}>
              <span>{t(r().decision.previewAllowed ? 'admin-attachments-sim-preview-yes' : 'admin-attachments-sim-preview-no')}</span>
            </Show>
            <span>{t('admin-attachments-sim-checked', { size: formatBytes(r().size) })}</span>
            <span>
              {r().matched.length === 0
                ? t('admin-attachments-sim-matched-none')
                : t('admin-attachments-sim-matched', { lists: r().matched.map((m) => t(`admin-attachments-list-${m}`)).join(', ') })}
            </span>
          </div>
        )}</Show>
      </div>
    </AdminSubsection>
  );
}

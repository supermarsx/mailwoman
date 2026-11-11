import { createSignal, For, onCleanup, Show, type JSX } from 'solid-js';
import { receiptPage, receiptTarget, type ReceiptSource, type ReceiptTarget, type RecipientReport } from '../api/messageReceipts.ts';
import { formatDateTime, loadCatalog, t } from '../i18n';
import { Icon } from './Icon.tsx';
import { SecurityPopup } from './SecurityPanel.tsx';
import './readerBadges.css';
import './sentReceiptBadge.css';

export function SentReceiptBadge(props: ReceiptSource & { emailId: string }): JSX.Element {
  void loadCatalog('sent-receipts');
  void loadCatalog('mail-dsn');
  const [open, setOpen] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [failed, setFailed] = createSignal(false);
  const [incomplete, setIncomplete] = createSignal(false);
  const [target, setTarget] = createSignal<ReceiptTarget>();
  const [reports, setReports] = createSignal<RecipientReport[]>([]);
  const [position, setPosition] = createSignal<number | null>(0);
  let request: AbortController | undefined;
  let button: HTMLButtonElement | undefined;
  onCleanup(() => request?.abort());

  async function load(): Promise<void> {
    const offset = position();
    if (busy() || offset === null) return;
    const controller = new AbortController();
    request = controller;
    const timeout = setTimeout(() => controller.abort(), 20_000);
    setBusy(true);
    setFailed(false);
    try {
      const original = target() ?? await receiptTarget(props, props.emailId, controller.signal);
      controller.signal.throwIfAborted();
      setTarget(original);
      const page = await receiptPage(props, original, offset, controller.signal);
      controller.signal.throwIfAborted();
      setReports(previous => [...previous, ...page.reports]);
      setPosition(page.next);
      setIncomplete(previous => previous || page.incomplete);
    } catch {
      if (request === controller) setFailed(true);
    } finally {
      clearTimeout(timeout);
      if (request === controller) {
        request = undefined;
        setBusy(false);
      }
    }
  }

  function close(): void {
    const pending = request;
    request = undefined;
    pending?.abort();
    setBusy(false);
    setOpen(false);
    button?.focus();
  }

  function status(address: string, kind: RecipientReport['kind']): JSX.Element {
    const latest = () => reports().filter(report => report.address === address && report.kind === kind)
      .reduce<RecipientReport | undefined>((current, report) => !current || report.at > current.at ? report : current, undefined);
    const requested = () => kind === 'read' ? target()?.readRequested : target()?.deliveryRequested;
    return (
      <Show when={latest()} fallback={t(requested() ? 'sent-receipts-awaiting' : 'sent-receipts-no-report')}>
        {report => (
          <>
            <strong>{t(kind === 'delivery' ? `dsn-action-${report().status}` : `sent-receipts-mdn-${report().status}`)}</strong>
            <Show when={report().code}><code> {report().code}</code></Show>
            <time dateTime={new Date(report().at).toISOString()}>{formatDateTime(report().at)}</time>
          </>
        )}
      </Show>
    );
  }
  const showRead = () => target()?.readRequested || reports().some(report => report.kind === 'read');
  const showDelivery = () => target()?.deliveryRequested || reports().some(report => report.kind === 'delivery');
  return (
    <>
      <button type="button" ref={button} class="mw-badge mw-badge--keep-label" aria-haspopup="dialog"
        aria-expanded={open()} onClick={() => { setOpen(true); if (position() === 0 || failed()) void load(); }}>
        <Icon name="receipt" /> <span class="mw-badge__label">{t('sent-receipts-button')}</span>
      </button>
      <Show when={open()}>
        <SecurityPopup title={t('sent-receipts-title')} onClose={close}>
          <div class="sent-receipts">
            <p>{t('sent-receipts-explanation')}</p>
            <Show when={busy()}><p role="status">{t('sent-receipts-loading')}</p></Show>
            <Show when={failed()}><p role="alert">{t('sent-receipts-failed')}</p></Show>
            <Show when={incomplete()}><p role="status">{t('sent-receipts-incomplete')}</p></Show>
            <Show when={target() && !target()!.messageId}><p>{t('sent-receipts-missing-id')}</p></Show>
            <Show when={target()?.messageId && (showRead() || showDelivery())}>
              <div class="sent-receipts__table">
                <table>
                  <thead><tr>
                    <th scope="col">{t('sent-receipts-recipient')}</th>
                    <Show when={showRead()}><th scope="col">{t('sent-receipts-read')}</th></Show>
                    <Show when={showDelivery()}><th scope="col">{t('sent-receipts-delivery')}</th></Show>
                  </tr></thead>
                  <tbody><For each={target()?.recipients}>{address => <tr>
                    <th scope="row"><bdi>{address}</bdi></th>
                    <Show when={showRead()}><td>{status(address, 'read')}</td></Show>
                    <Show when={showDelivery()}><td>{status(address, 'delivery')}</td></Show>
                  </tr>}</For></tbody>
                </table>
              </div>
            </Show>
            <Show when={target()?.messageId && !showRead() && !showDelivery() && !busy() && !failed()}>
              <p>{t('sent-receipts-none')}</p>
            </Show>
            <Show when={position() !== null && (target()?.messageId || failed())}>
              <button type="button" class="btn" disabled={busy()} onClick={() => void load()}>
                {t(failed() ? 'common-retry' : 'sent-receipts-more')}
              </button>
            </Show>
          </div>
        </SecurityPopup>
      </Show>
    </>
  );
}

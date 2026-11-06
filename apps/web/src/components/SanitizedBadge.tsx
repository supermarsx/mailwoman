import {createSignal, createUniqueId, Show, type JSX} from 'solid-js';
import {t} from '../i18n';
import {Icon} from './Icon.tsx';
import {SecurityPopup} from './SecurityPanel.tsx';
import './readerBadges.css';

export interface SanitizedReport {blockedCount: number; trackerCount: number; hostCount: number}

/**
 * What the HTML sanitizer did to the open message, as a badge in the reader's metadata row. `blocked`: the body
 * could not be shown as HTML; `sanitized`: sanitized HTML is shown; `clean`: nothing was changed. The badge opens a
 * small dialog with the counts the sanitizer reported; the optional action (show sanitized HTML / show safe text) sits
 * beside the label, where the old notice bar had it.
 */
export function SanitizedBadge(props: {
  state: 'blocked' | 'sanitized' | 'clean';
  report: SanitizedReport;
  action?: {icon: 'eye' | 'shield'; label: string; disabled?: boolean; onClick: () => void} | undefined;
}): JSX.Element {
  const [open, setOpen] = createSignal(false);
  const id = `mw-sanitized-${createUniqueId()}`;
  let trigger: HTMLButtonElement | undefined;
  const close = (): void => { setOpen(false); trigger?.focus(); };
  const label = (): string => t(props.state === 'blocked' ? 'privacy-body-blocked' : props.state === 'sanitized' ? 'privacy-body-sanitized-shown' : 'reader-sanitized-clean');
  const short = (): string => t(props.state === 'blocked' ? 'reader-sanitized-short-blocked' : props.state === 'sanitized' ? 'reader-sanitized-short' : 'reader-sanitized-short-clean');
  const tone = (): string => (props.state === 'blocked' ? 'warning' : props.state === 'sanitized' ? 'neutral' : 'good');
  const summary = (): string => props.state === 'clean' ? t('reader-sanitized-clean-detail') : `${t(props.state === 'blocked' ? 'privacy-body-blocked-detail' : 'reader-sanitized-detail')} ${t('reader-sanitized-counts', {blocked: props.report.blockedCount, trackers: props.report.trackerCount})}`;
  return <span class="mw-badge mw-badge--group mw-badge--sanitized" data-testid="reader-body-blocked" data-state={props.state} data-tone={tone()} role="group" aria-label={label()}>
    <button ref={trigger} type="button" class="mw-badge__part" aria-haspopup="dialog" aria-expanded={open()} aria-controls={`${id}-dialog`} aria-label={label()} data-tooltip={summary()} onClick={() => setOpen(v => !v)}>
      <Icon name={props.state === 'clean' ? 'check' : 'shield'}/><span class="mw-badge__label">{short()}</span>
    </button>
    <Show when={props.action}>{action => <button type="button" class="mw-badge__part mw-badge__part--action" aria-label={action().label} data-tooltip={action().label} disabled={action().disabled} onClick={() => action().onClick()}><Icon name={action().icon}/></button>}</Show>
    <Show when={open()}>
      <SecurityPopup id={`${id}-dialog`} title={label()} onClose={close}>
        <div class="sanitized-detail" data-testid="reader-sanitized-detail">
          <p>{props.state === 'clean' ? t('reader-sanitized-clean-detail') : t(props.state === 'blocked' ? 'privacy-body-blocked-detail' : 'reader-sanitized-detail')}</p>
          <dl>
            <div><dt>{t('reader-sanitized-remote')}</dt><dd>{props.report.blockedCount}</dd></div>
            <div><dt>{t('reader-sanitized-trackers')}</dt><dd>{props.report.trackerCount}</dd></div>
            <div><dt>{t('reader-sanitized-hosts')}</dt><dd>{props.report.hostCount}</dd></div>
          </dl>
          <p class="sanitized-detail__note">{t('reader-sanitized-always')}</p>
        </div>
      </SecurityPopup>
    </Show>
  </span>;
}

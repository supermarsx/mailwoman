import { Show, type JSX } from 'solid-js';
import { t } from '../i18n';
import * as css from './MessageLoading.css.ts';
import * as a11y from './mailA11y.css.ts';

export function MessageLoading(props: { variant: string; delayMs: number }): JSX.Element {
  const delay = () => Number.isFinite(props.delayMs) ? Math.max(0, Math.min(1000, props.delayMs)) : 120;
  return <div class={props.variant === 'none' ? a11y.srOnly : css.panel} role="status" aria-live="polite" style={{ '--reader-loading-delay': `${delay()}ms` }}>
    <span>{t('mail-sanitizing')}</span>
    <Show when={props.variant === 'skeleton'}><div class={css.lines} aria-hidden="true"><span class={css.line} /><span class={css.line} /><span class={css.line} /></div></Show>
  </div>;
}

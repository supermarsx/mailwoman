import type { JSX } from 'solid-js';
import { t } from '../i18n';
import { InlineLoading } from './InlineLoading.tsx';

/** The reader body's wait: one inline label, drawn after the delay, nothing else. */
export function MessageLoading(props: { variant: string; delayMs: number; message?: string }): JSX.Element {
  const delay = () => Number.isFinite(props.delayMs) ? Math.max(0, Math.min(1000, props.delayMs)) : 150;
  // 'none' keeps the announcement but never draws anything.
  return <InlineLoading message={props.message ?? t('mail-sanitizing')} delayMs={props.variant === 'none' ? 3_600_000 : delay()} />;
}

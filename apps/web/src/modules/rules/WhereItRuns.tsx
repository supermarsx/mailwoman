// "Where it runs" indicator (audit #1): a rule runs either as server-uploaded
// Sieve (ManageSieve PUTSCRIPT) or engine-side at ingest. Surfaces the frozen
// `MailRule.runsAt` so the user understands where their filter executes.

import { Show, type JSX } from 'solid-js';
import { t } from '../../i18n';
import * as css from './styles.css.ts';

export interface WhereItRunsProps {
  runsAt: 'server-sieve' | 'engine';
}

export function WhereItRuns(props: WhereItRunsProps): JSX.Element {
  return (
    <p class={css.prose}>
      <Show
        when={props.runsAt === 'server-sieve'}
        fallback={
          <>
            <span class={css.badge}>{t('rules-runs-engine')}</span> {t('rules-runs-engine-detail')}
          </>
        }
      >
        <span class={`${css.badge} ${css.badgeServer}`}>{t('rules-runs-server')}</span>{' '}
        {t('rules-runs-server-detail')}
      </Show>
    </p>
  );
}

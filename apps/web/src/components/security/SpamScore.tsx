import { createResource, For, Show, onCleanup, type JSX } from 'solid-js';
import { fetchMailHeaders } from '../../api/mailHeaders.ts';
import { parseSpamScores } from './spamScore.ts';
import { t } from '../../i18n';
import { Icon } from '../Icon.tsx';
import './spamScore.css';

export function SpamScore(props: { url: string; onHeaders?: (raw:string|null,url:string)=>void }): JSX.Element {
  let current:AbortController|undefined;
  onCleanup(()=>current?.abort());
  const [scores]=createResource(()=>props.url,async url=>{
    current?.abort();const abort=new AbortController();current=abort;
    props.onHeaders?.(null,url);
    const timeout=setTimeout(()=>abort.abort(),10000);
    try{const raw=await fetchMailHeaders(url,abort.signal);if(current===abort&&!abort.signal.aborted)props.onHeaders?.(raw,url);return parseSpamScores(raw);}catch{return [];}finally{clearTimeout(timeout);}
  });
  const ready=()=>scores.state==='ready'?scores.latest:[];
  return <div class="spam-score" aria-label={t('mail-spam-score')}>
    <Show when={ready()?.length} fallback={<span class="spam-score__empty">{t(scores.loading?'mail-spam-score-loading':'mail-spam-score-unavailable')}</span>}>
      <For each={ready()}>{score=><details class="spam-score__item"><summary><Icon name="shield"/><span class="spam-score__source" data-tooltip={score.source}>{score.source}</span> <strong>{score.score}{score.threshold!==null?` / ${score.threshold}`:''}</strong></summary>
        <div class="spam-score__details"><p>{t('mail-spam-score-unverified')}</p><p>{t(score.spam===true?'mail-spam-score-positive':score.spam===false?'mail-spam-score-negative':'mail-spam-score-unknown')}</p><Show when={score.rules.length}><ul><For each={score.rules}>{rule=><li><code>{rule}</code></li>}</For></ul></Show></div>
      </details>}</For>
    </Show>
  </div>;
}

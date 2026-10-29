import { createResource, createUniqueId, For, Show, onCleanup, onMount, type JSX } from 'solid-js';
import { fetchMailHeaders } from '../../api/mailHeaders.ts';
import { parseSpamScores, spamBand, type SpamScore as Score } from './spamScore.ts';
import { loadCatalog, t } from '../../i18n';
import { Icon } from '../Icon.tsx';
import { RibbonPopover } from '../compose/ribbonParts.tsx';
import './spamScore.css';

/** A rule's points with their sign, so a negative (ham) contribution never reads as a positive one. */
export function signedPoints(value: number): string {
  const text = String(Math.abs(value));
  return value > 0 ? `+${text}` : value < 0 ? `−${text}` : text;
}

/**
 * The reader's header-reported spam scores: one compact chip per scanner (score
 * and a ham/neutral/spam band). The rule breakdown opens in the same popover as
 * the composer's security chip, never inline.
 */
export function SpamScore(props: { url: string; onHeaders?: (raw:string|null,url:string)=>void }): JSX.Element {
  onMount(() => void loadCatalog('spam-score'));
  let current:AbortController|undefined;
  onCleanup(()=>current?.abort());
  const [scores]=createResource(()=>props.url,async url=>{
    current?.abort();const abort=new AbortController();current=abort;
    props.onHeaders?.(null,url);
    const timeout=setTimeout(()=>abort.abort(),10000);
    try{const raw=await fetchMailHeaders(url,abort.signal);if(current===abort&&!abort.signal.aborted)props.onHeaders?.(raw,url);return parseSpamScores(raw);}catch{return [];}finally{clearTimeout(timeout);}
  });
  const ready=()=>scores.state==='ready'?scores.latest:[];
  return <div class="spam-score" role="group" aria-label={t('mail-spam-score')}>
    <Show when={ready()?.length} fallback={<span class="spam-score__empty">{t(scores.loading?'mail-spam-score-loading':'mail-spam-score-unavailable')}</span>}>
      <For each={ready()}>{score=><SpamScoreChip score={score}/>}</For>
    </Show>
  </div>;
}

export function SpamScoreChip(props: { score: Score }): JSX.Element {
  const titleId = `mw-spam-score-${createUniqueId()}`;
  const band = () => spamBand(props.score);
  const bandText = () => t(`spam-score-band-${band()}`);
  const label = () => props.score.threshold !== null
    ? t('spam-score-chip-label', { source: props.score.source, score: String(props.score.score), threshold: String(props.score.threshold), band: bandText() })
    : t('spam-score-chip-label-no-threshold', { source: props.score.source, score: String(props.score.score), band: bandText() });
  return <RibbonPopover
    label={label()}
    ariaLabel={label()}
    labelledBy={titleId}
    focusDialog
    testId="spam-score-item"
    buttonClass={`spam-score__chip spam-score__chip--${band()}`}
    content={<span class="spam-score__chip-body" data-band={band()}>
      <Icon name="shield"/><span class="spam-score__source">{props.score.source}</span> <strong><span class="spam-score__value">{String(props.score.score)}</span><Show when={props.score.threshold !== null}><span class="spam-score__of">{` / ${props.score.threshold}`}</span></Show></strong>
    </span>}
  >
    <SpamScoreBreakdown score={props.score} titleId={titleId}/>
  </RibbonPopover>;
}

/** The rule table: one layout for SpamAssassin rules and Rspamd symbols, largest contribution first. */
export function SpamScoreBreakdown(props: { score: Score; titleId: string }): JSX.Element {
  const band = () => spamBand(props.score);
  return <div class="spam-score__sheet" data-testid="spam-score-breakdown">
    <h3 id={props.titleId} class="spam-score__title">{t('spam-score-title', { source: props.score.source })}</h3>
    <dl class="spam-score__summary">
      <div><dt>{t('spam-score-total')}</dt><dd><strong>{String(props.score.score)}</strong></dd></div>
      <div><dt>{t('spam-score-threshold')}</dt><dd>{props.score.threshold !== null ? String(props.score.threshold) : t('spam-score-threshold-missing')}</dd></div>
      <div><dt>{t('spam-score-band')}</dt><dd><span class={`spam-score__band spam-score__band--${band()}`}>{t(`spam-score-band-${band()}`)}</span></dd></div>
    </dl>
    <p class="spam-score__note">{t(props.score.spam===true?'mail-spam-score-positive':props.score.spam===false?'mail-spam-score-negative':'mail-spam-score-unknown')}</p>
    <Show when={props.score.rules.length} fallback={<p class="spam-score__note">{t('spam-score-no-rules')}</p>}>
      <table class="spam-score__table">
        <caption>{t('spam-score-caption')}</caption>
        <thead><tr><th scope="col">{t('spam-score-col-rule')}</th><th scope="col" class="spam-score__num">{t('spam-score-col-points')}</th><th scope="col">{t('spam-score-col-description')}</th></tr></thead>
        <tbody>
          <For each={props.score.rules}>{rule=><tr>
            <th scope="row"><code>{rule.name}</code></th>
            <td class="spam-score__num" data-sign={rule.score === null ? 'none' : rule.score > 0 ? 'plus' : rule.score < 0 ? 'minus' : 'zero'}>{rule.score === null ? t('spam-score-points-missing') : signedPoints(rule.score)}</td>
            <td>{rule.description}</td>
          </tr>}</For>
        </tbody>
        <tfoot><tr><th scope="row">{t('spam-score-total')}</th><td class="spam-score__num"><strong>{String(props.score.score)}</strong></td><td>{props.score.threshold !== null ? t('spam-score-of-threshold', { threshold: String(props.score.threshold) }) : ''}</td></tr></tfoot>
      </table>
    </Show>
    <p class="spam-score__note spam-score__note--limit">{t('mail-spam-score-unverified')} {t('spam-score-not-scanned')}</p>
  </div>;
}

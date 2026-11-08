import { createResource, createSignal, createUniqueId, For, Show, onCleanup, onMount, type JSX } from 'solid-js';
import { fetchMailHeaders } from '../../api/mailHeaders.ts';
import { parseSpamScores, spamBand, type SpamScore as Score } from './spamScore.ts';
import { loadCatalog, t } from '../../i18n';
import { Icon } from '../Icon.tsx';
import { badgeMode, badgeVisible } from '../../state/readerBadges.ts';
import { SecurityPopup } from '../SecurityPanel.tsx';
import '../readerBadges.css';
import './spamScore.css';

/** A rule's points with their sign, so a negative (ham) contribution never reads as a positive one. */
export function signedPoints(value: number): string {
  const text = String(Math.abs(value));
  return value > 0 ? `+${text}` : value < 0 ? `−${text}` : text;
}

/**
 * The reader's header-reported spam scores: one compact chip per scanner (score
 * and a ham/neutral/spam band). The rule breakdown opens in the same dialog as
 * the reader's security verification, never inline.
 */
/** `onHeaders` gets null when a fetch starts, the raw header block when it
 *  arrives and '' when it fails or times out (t110: the reader's notice bars wait
 *  for a settled answer before they appear). */
export function SpamScore(props: { url: string; onHeaders?: (raw:string|null,url:string)=>void; /** Wraps the chips (the badge's right-click menu). */menu?: (chips: JSX.Element)=>JSX.Element }): JSX.Element {
  onMount(() => void loadCatalog('spam-score'));
  let current:AbortController|undefined;
  onCleanup(()=>current?.abort());
  const [scores]=createResource(()=>props.url,async url=>{
    current?.abort();const abort=new AbortController();current=abort;
    props.onHeaders?.(null,url);
    const timeout=setTimeout(()=>abort.abort(),10000);
    try{const raw=await fetchMailHeaders(url,abort.signal);if(current===abort&&!abort.signal.aborted)props.onHeaders?.(raw,url);return parseSpamScores(raw);}catch{if(current===abort)props.onHeaders?.('',url);return [];}finally{clearTimeout(timeout);}
  });
  // The header is always fetched (the notice bars wait for it); `mail.badge.spam` only decides which chips show.
  const ready=()=>scores.state==='ready'?(scores.latest??[]).filter(score=>badgeVisible('spam',spamBand(score)!=='ham')):[];
  return <div class="spam-score" role="group" aria-label={t('mail-spam-score')}>
    <Show when={ready()?.length} fallback={<Show when={badgeMode('spam')==='always'}><span class="spam-score__empty">{t(scores.loading?'mail-spam-score-loading':'mail-spam-score-unavailable')}</span></Show>}>
      {(props.menu??(chips=>chips))(<For each={ready()}>{score=><SpamScoreChip score={score}/>}</For>)}
    </Show>
  </div>;
}

export function SpamScoreChip(props: { score: Score }): JSX.Element {
  const titleId = `mw-spam-score-${createUniqueId()}`;
  const dialogId = `${titleId}-dialog`;
  const [open, setOpen] = createSignal(false);
  let trigger: HTMLButtonElement | undefined;
  const close = (): void => { setOpen(false); trigger?.focus(); };
  const band = () => spamBand(props.score);
  const bandText = () => t(`spam-score-band-${band()}`);
  const label = () => props.score.threshold !== null
    ? t('spam-score-chip-label', { source: props.score.source, score: String(props.score.score), threshold: String(props.score.threshold), band: bandText() })
    : t('spam-score-chip-label-no-threshold', { source: props.score.source, score: String(props.score.score), band: bandText() });
  return <div data-testid="spam-score-item"><button ref={trigger} type="button"
    aria-label={label()} aria-haspopup="dialog" aria-expanded={open()} aria-controls={dialogId}
    class={`mw-badge spam-score__chip spam-score__chip--${band()}`} data-tone={band()==='ham'?'good':band()==='spam'?'bad':'warning'} onClick={()=>setOpen(value=>!value)}>
    <span class="spam-score__chip-body" data-band={band()}>
      <Icon name="shield"/><span class="spam-score__source">{props.score.source}</span> <strong><span class="spam-score__value">{String(props.score.score)}</span><Show when={props.score.threshold !== null}><span class="spam-score__of">{` / ${props.score.threshold}`}</span></Show></strong>
    </span>
  </button><Show when={open()}><SecurityPopup id={dialogId} title={t('spam-score-title', { source: props.score.source })} labelledBy={titleId} onClose={close}>
    <SpamScoreBreakdown score={props.score}/>
  </SecurityPopup></Show></div>;
}

/** The rule table: one layout for SpamAssassin rules and Rspamd symbols, largest contribution first. */
export function SpamScoreBreakdown(props: { score: Score; titleId?: string }): JSX.Element {
  const band = () => spamBand(props.score);
  const [copyState,setCopyState]=createSignal<'idle'|'copying'|'copied'|'failed'>('idle');
  let alive=true;onCleanup(()=>{alive=false;});
  async function copy():Promise<void>{
    if(copyState()==='copying')return;
    const score=props.score;
    const text=[
      t('spam-score-title',{source:score.source}),
      `${t('spam-score-total')}: ${score.score}`,
      `${t('spam-score-threshold')}: ${score.threshold??t('spam-score-threshold-missing')}`,
      `${t('spam-score-band')}: ${t(`spam-score-band-${spamBand(score)}`)}`,
      t(score.spam===true?'mail-spam-score-positive':score.spam===false?'mail-spam-score-negative':'mail-spam-score-unknown'),
      '',
      score.rules.length?[t('spam-score-col-rule'),t('spam-score-col-points'),t('spam-score-col-description')].join('\t'):t('spam-score-no-rules'),
      ...score.rules.map(rule=>[rule.name,rule.score===null?t('spam-score-points-missing'):signedPoints(rule.score),rule.description].join('\t')),
      '',t('mail-spam-score-unverified'),t('spam-score-not-scanned'),
    ].join('\n');
    setCopyState('copying');
    try{await navigator.clipboard.writeText(text);if(alive&&props.score===score)setCopyState('copied');}
    catch{if(alive&&props.score===score)setCopyState('failed');}
  }
  return <div class="spam-score__sheet" data-testid="spam-score-breakdown">
    <Show when={props.titleId}><h3 id={props.titleId} class="spam-score__title">{t('spam-score-title', { source: props.score.source })}</h3></Show>
    <div class="spam-score__actions">
      <Show when={copyState()==='copied'||copyState()==='failed'}><span role="status">{t(copyState()==='copied'?'spam-score-copied':'spam-score-copy-failed')}</span></Show>
      <button type="button" class="btn btn--ghost" disabled={copyState()==='copying'} onClick={()=>void copy()}><Icon name="clone"/>{t('spam-score-copy')}</button>
    </div>
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

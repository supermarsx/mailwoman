import { createSignal, createEffect, on, For, Show, type JSX } from 'solid-js';
import { t, isolate, loadCatalog } from '../../i18n';
import { RawEditor } from './RawEditor.tsx';
import { WhereItRuns } from './WhereItRuns.tsx';
import type { SieveApi, SieveCapability, ScriptRecord } from './sieve-api.ts';
import * as css from './styles.css.ts';

export function ScriptsPanel(props: { api: SieveApi }): JSX.Element {
  const [cap, setCap] = createSignal<SieveCapability | null>(null);
  const [scripts, setScripts] = createSignal<ScriptRecord[]>([]);
  const [state, setState] = createSignal('');
  const [selected, setSelected] = createSignal<string | null>(null);
  const [name, setName] = createSignal('');
  const [source, setSource] = createSignal('keep;\n');
  const [loading, setLoading] = createSignal(true);
  const [busy, setBusy] = createSignal(false);
  const [message, setMessage] = createSignal('');
  const [failure, setFailure] = createSignal('');
  let generation = 0;
  const mayEdit = (): boolean => cap() !== null && (cap()?.['urn:mailwoman:policy']?.rawEdit ?? true);
  const active = (): boolean => scripts().some((s) => s.id === selected() && s.isActive);
  async function choose(script: ScriptRecord | null, api = props.api): Promise<void> {
    const mine = ++generation;
    setSelected(script?.id ?? null); setName(script?.name ?? ''); setFailure(''); setMessage('');
    if (!script) { setSource('keep;\n'); return; }
    setBusy(true);
    try { const text = await api.source(script); if (mine === generation) setSource(text); }
    catch (error) { if (mine === generation) setFailure(error instanceof Error ? error.message : String(error)); }
    finally { if (mine === generation) setBusy(false); }
  }
  async function refresh(prefer: string | null = selected(), api = props.api): Promise<void> {
    const list = await api.get(); if (api !== props.api) return; setScripts(list.list); setState(list.state);
    await choose(list.list.find((s) => s.id === prefer) ?? list.list.find((s) => s.isActive) ?? list.list[0] ?? null, api);
  }
  createEffect(on(() => props.api, (api) => {
    const mine = ++generation; setLoading(true); setFailure(''); setCap(null); setScripts([]); setSelected(null); setSource(''); setState(''); setBusy(false);
    void (async () => {
      await loadCatalog('rules');
      try {
        const capability = await api.connect(); if (mine !== generation) return;
        setCap(capability);
        if (capability) await refresh(null, api);
      } catch (error) { if (api === props.api) setFailure(error instanceof Error ? error.message : String(error)); }
      finally { if (api === props.api) setLoading(false); }
    })();
  }));
  async function action(work: () => Promise<void>): Promise<void> {
    const api = props.api;
    if (busy()) return; setBusy(true); setFailure(''); setMessage('');
    try { await work(); } catch (error) { if (api === props.api) setFailure(error instanceof Error ? error.message : String(error)); }
    finally { if (api === props.api) setBusy(false); }
  }
  async function save(): Promise<void> {
    const api = props.api;
    await api.save(selected(), name(), source(), state(), false);
    if (api !== props.api) return;
    await refresh(undefined, api); if (api === props.api) setMessage(t('rules-script-saved'));
  }
  function exportScript(): void {
    const url = URL.createObjectURL(new Blob([source()], { type: 'application/sieve' }));
    const anchor = document.createElement('a'); anchor.href = url; anchor.download = `${name() || 'script'}.sieve`; anchor.click();
    URL.revokeObjectURL(url);
  }
  return <div class={css.panel} aria-busy={busy() || loading()}>
    <Show when={loading()}><p role="status">{t('common-loading')}</p></Show>
    <Show when={failure()}><p role="alert">{failure()}</p></Show>
    <Show when={message()}><p role="status">{message()}</p></Show>
    <Show when={!loading() && cap() === null && !failure()}><p>{t('rules-unavailable')}</p></Show>
    <Show when={cap()}>{(capability) => <>
      <WhereItRuns runsAt={capability()['urn:mailwoman:policy']?.runsAt ?? 'server-sieve'} />
      <Show when={mayEdit()}><label class={css.label}>{t('rules-script-template')}
        <select class={css.input} disabled={busy()} value="" onChange={(event) => {
          const kind = event.currentTarget.value; if (!kind) return;
          void choose(null); setName(kind);
          setSource(kind === 'archive' ? 'require ["fileinto"];\nif header :contains "subject" "newsletter" { fileinto "Archive"; }\n'
            : kind === 'flag' ? 'require ["imap4flags"];\nif header :contains "subject" "important" { addflag "\\\\Flagged"; }\n'
            : 'require ["vacation"];\nvacation :days 7 "Away from my inbox.";\n');
        }}>
          <option value="">{t('rules-script-new')}</option>
          <Show when={capability().sieveExtensions.includes('fileinto')}><option value="archive">{t('rules-act-archive')}</option></Show>
          <Show when={capability().sieveExtensions.includes('imap4flags')}><option value="flag">{t('rules-act-tag')}</option></Show>
          <Show when={capability().sieveExtensions.includes('vacation')}><option value="vacation">{t('rules-vacation-title')}</option></Show>
        </select>
      </label></Show>
      <ul class={css.list} aria-label={t('rules-scripts-label')}>
        <For each={scripts()}>{(script) => <li class={css.ruleRow}>
          <button type="button" class={css.btn} disabled={busy()} aria-pressed={selected() === script.id} onClick={() => void choose(script)}>{isolate(script.name)}</button>
          <Show when={script.isActive}><span class={css.badge}>{t('rules-script-active')}</span></Show>
        </li>}</For>
      </ul>
      <div class={css.row}>
        <button type="button" class={css.btn} disabled={!mayEdit() || busy()} onClick={() => void choose(null)}>{t('rules-script-new')}</button>
        <button type="button" class={css.btn} disabled={busy()} onClick={exportScript}>{t('rules-script-export')}</button>
        <Show when={mayEdit()}><label class={css.label}>{t('rules-script-import')}
          <input type="file" accept=".sieve,application/sieve" disabled={busy()} onChange={(event) => {
            const file = event.currentTarget.files?.[0]; if (!file) return;
            void action(async () => {
              if (capability().maxSizeScript !== null && file.size > capability().maxSizeScript!) throw new Error(t('rules-script-too-large'));
              const text = await file.text(); await choose(null); setName(file.name.replace(/\.sieve$/i, '')); setSource(text);
            });
          }} />
        </label></Show>
      </div>
      <label class={css.label}>{t('rules-name-label')}
        <input class={css.input} value={name()} disabled={!mayEdit() || busy()} onInput={(event) => setName(event.currentTarget.value)} />
      </label>
      <Show when={!mayEdit()}><p class={css.prose}>{t('rules-script-read-only')}</p></Show>
      <RawEditor source={source()} onInput={setSource} readOnly={!mayEdit() || busy()} />
      <div class={css.row}>
        <button type="button" class={css.btn} disabled={!mayEdit() || busy()} onClick={() => void action(async () => {
          const error = await props.api.validate(source()); if (error) setFailure(error.description ?? error.type); else setMessage(t('rules-lint-clean'));
        })}>{t('rules-script-validate')}</button>
        <button type="button" class={css.btnPrimary} disabled={!mayEdit() || busy() || !name().trim()} onClick={() => void action(save)}>{t('common-save')}</button>
        <button type="button" class={css.btn} disabled={busy() || selected() === null} onClick={() => void action(async () => {
          await props.api.activate(active() ? null : selected(), state()); await refresh();
        })}>{active() ? t('rules-script-deactivate') : t('rules-script-activate')}</button>
        <button type="button" class={css.btnDanger} disabled={busy() || selected() === null || active()} onClick={() => void action(async () => {
          const id = selected(); if (id) { await props.api.remove(id, state()); await refresh(null); }
        })}>{t('common-delete')}</button>
      </div>
    </>}</Show>
  </div>;
}

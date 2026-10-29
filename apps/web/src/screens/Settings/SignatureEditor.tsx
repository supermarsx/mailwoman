import { preferenceAccount, preferenceAccountIsCurrent } from '../../api/accountPreferenceLifecycle.ts';
import { createSignal, lazy, Match, onCleanup, Show, Suspense, Switch, type JSX } from 'solid-js';
import type { Signature } from './types.ts';
import type { SettingsService } from './service.ts';
import { t, isolate } from '../../i18n';
import { toast } from '../../components/toast/index.ts';
import { useEditorGuard } from './entityEditor.tsx';
import { escapeHtml } from '../../viewers/sandbox.ts';
import { BodyModeGroup, EditorRibbon, createEditorRibbon, type BodyMode, type EditorTab } from '../../components/compose/EditorRibbon.tsx';
import type { EditorGroup, RichTextApi } from '../../components/compose/RichTextEditor.tsx';
import * as css from './styles.css.ts';
import * as layout from './editor.css.ts';
import * as sig from './signatureEditor.css.ts';

// The signature editor (t49) is the composer's editor: the same `RichTextEditor`
// and HTML source editor, under the same `EditorRibbon`, with the groups that
// apply to a signature (no recipients, attachments, security or delivery, and
// no find and replace). Images are embedded in the HTML (a signature has no
// uploads); the composer turns them into inline images of each message.
const RichEditor = lazy(() => import('../../components/compose/RichTextEditor.tsx'));
const SourceEditor = lazy(() => import('../../components/compose/HtmlSourceEditor.tsx'));
const plainToHtml = (text: string): string => `<p>${escapeHtml(text).replace(/\n/g, '<br>')}</p>`;
const NO_OWNED: ReadonlySet<string> = new Set();
/** The server's limit on a signature's HTML (`prefs_routes.rs`), in UTF-8 bytes. */
export const SIGNATURE_HTML_MAX = 32 * 1024;
const OMIT: readonly EditorGroup[] = ['editing'];
const utf8 = (value: string): number => new TextEncoder().encode(value).length;

export function SignatureEditor(props: { initial?: Signature; service: SettingsService; onSaved: () => void | Promise<void>; onCancel: () => void | Promise<void> }): JSX.Element {
  const owner = preferenceAccount();
  let disposed = false;
  onCleanup(() => {
    disposed = true;
  });
  const current = (): boolean => !disposed && (owner === null || preferenceAccountIsCurrent(owner));
  const [draft, setDraft] = createSignal<Signature>(structuredClone(props.initial ?? { name: '', body: '', isDefault: false }));
  const [mode, setMode] = createSignal<BodyMode>(props.initial?.html ? 'rich' : 'plain');
  const plain = (): string => draft().text ?? draft().body;
  /** What a save sends: the HTML only while the format is not plain. Dirty = this differs from the last save. */
  const payload = (): Signature => ({ ...draft(), name: draft().name.trim(), body: plain(), text: plain(), html: mode() === 'plain' ? null : (draft().html ?? '') });
  const snapshot = (): string => JSON.stringify(payload());
  const [baseline, setBaseline] = createSignal(snapshot());
  const [busy, setBusy] = createSignal(false);
  const [fieldError, setFieldError] = createSignal('');
  const [imageError, setImageError] = createSignal('');
  const guard = useEditorGuard(() => snapshot() !== baseline(), busy);
  const ribbon = createEditorRibbon();
  const [editorApi, setEditorApi] = createSignal<RichTextApi | null>(null);
  // The editor reports its first serialisation as it mounts. That is the stored
  // HTML read through the editor's schema, not an edit, so while nothing has
  // been changed it moves the baseline with it (opening is never "unsaved").
  let settling = true;

  function changeMode(next: BodyMode): void {
    if (next === mode()) return;
    const clean = snapshot() === baseline();
    if (next !== 'rich' && ribbon.tab() === 'table') ribbon.setTab('format');
    if (mode() === 'plain') setDraft((value) => ({ ...value, html: value.html ?? plainToHtml(plain()) }));
    setEditorApi(null);
    // Rich and HTML source are two views of the same HTML; plain text is a change of format.
    settling = clean && next !== 'plain' && mode() !== 'plain';
    setMode(next);
  }
  function settle(): void {
    if (settling) setBaseline(snapshot());
  }
  /** The rich editor and the source editor both report HTML and its plain projection.
   *  The source editor's typed source is what is stored (sanitised by the server). */
  function changeHtml(html: string, text: string, source?: string): void {
    setDraft((value) => ({ ...value, html: source ?? html, ...(!value.textEdited ? { text, body: text } : {}) }));
    settle();
  }
  function ready(api: RichTextApi): void {
    setEditorApi(api);
    settling = false;
  }
  function changePlain(value: string): void {
    settling = false;
    setDraft((cur) => ({ ...cur, body: value, text: value, textEdited: true }));
  }

  async function save(): Promise<void> {
    if (busy()) return;
    if (!current()) {
      setFieldError(t('settings-sig-account-changed'));
      return;
    }
    if (draft().name.trim() === '') {
      setFieldError(t('settings-sig-error-name'));
      return;
    }
    const value = payload();
    if (value.html !== null && value.html !== undefined && utf8(value.html) > SIGNATURE_HTML_MAX) {
      setFieldError(t('settings-sig-too-large'));
      return;
    }
    setFieldError('');
    setBusy(true);
    const submitted = JSON.stringify(value);
    try {
      await props.service.upsertSignature(value);
      if (!current()) return;
      setBaseline(submitted);
      if (snapshot() !== submitted) return;
      toast.success(t('settings-saved'));
      await props.onSaved();
    } catch {
      toast.error(t('settings-sig-error-generic'));
    } finally {
      setBusy(false);
    }
  }

  const embedded = {
    // Room left for images: the limit less the HTML there is now.
    fits: (bytes: number): boolean => utf8(draft().html ?? '') + bytes <= SIGNATURE_HTML_MAX,
    refused: (reason: 'type' | 'size', name: string): void => {
      setImageError(t(reason === 'type' ? 'settings-sig-image-type' : 'settings-sig-image-size', { name: isolate(name) }));
    },
  };
  const visible = (): readonly EditorTab[] => (mode() === 'rich' ? ['format', 'insert', 'table'] : ['format']);
  const loading = (): JSX.Element => <p role="status" class={sig.pending}>{t('settings-sig-editor-loading')}</p>;
  const plainField = (label: string, testId: string): JSX.Element => (
    <textarea class={css.textarea} aria-label={label} data-testid={testId} value={plain()} onInput={(e) => changePlain(e.currentTarget.value)} />
  );

  return (
    <section class={css.section} aria-label={t(props.initial ? 'settings-sig-edit-title' : 'settings-sig-new')}>
      <h2 class={css.heading}>{t(props.initial ? 'settings-sig-edit-title' : 'settings-sig-new')}</h2>
      <form onSubmit={(event) => { event.preventDefault(); void save(); }} class={css.field} data-testid="signature-editor">
        <fieldset disabled={busy()} class={layout.fields}>
          <label class={css.field}>
            <span class={css.label}>{t('settings-sig-name-label')}</span>
            <input class={css.input} aria-label={t('settings-sig-name-label')} readOnly={props.initial !== undefined} value={draft().name} onInput={(e) => setDraft({ ...draft(), name: e.currentTarget.value })} />
          </label>
          <div inert={busy()}>
            <EditorRibbon
              state={ribbon}
              visible={visible}
              history={editorApi}
              showHistory={() => mode() === 'rich'}
              tabs={[
                { id: 'format', after: () => <BodyModeGroup label={t('settings-sig-format')} value={mode} onChange={changeMode} testId="signature-group-format" selectTestId="signature-format" /> },
                { id: 'insert' },
                { id: 'table' },
              ]}
            />
            <div class={sig.body} data-testid="signature-body">
              <Switch>
                <Match when={mode() === 'plain'}>{plainField(t('settings-sig-body-label'), 'signature-plain')}</Match>
                <Match when={mode() === 'rich'}>
                  <Suspense fallback={loading()}>
                    <RichEditor
                      externalHistory
                      ribbonSlots={ribbon.slots}
                      omitGroups={OMIT}
                      initialHtml={draft().html ?? ''}
                      ariaLabel={t('settings-sig-rich-body')}
                      embedded={embedded}
                      onChange={(html, text) => changeHtml(html, text)}
                      onReady={ready}
                    />
                  </Suspense>
                </Match>
                <Match when={mode() === 'html'}>
                  <Suspense fallback={loading()}>
                    <SourceEditor initialHtml={draft().html ?? ''} owned={() => NO_OWNED} embedded onChange={changeHtml} onReady={ready} />
                  </Suspense>
                </Match>
              </Switch>
            </div>
          </div>
          <Show when={imageError()}><p class={css.error} role="alert">{imageError()}</p></Show>
          <Show when={mode() !== 'plain'}>
            <p class={css.prose}>{t('settings-sig-html-help')} {t('settings-sig-images-help')}</p>
            <label class={css.field}>
              <span class={css.label}>{t('settings-sig-plain-alternative')}</span>
              {plainField(t('settings-sig-plain-alternative'), 'signature-plain-alternative')}
            </label>
            <p class={css.prose}>{t('settings-sig-plain-help')}</p>
          </Show>
          <label class={css.check}>
            <input class={css.checkbox} type="checkbox" aria-label={t('settings-sig-default-label')} checked={draft().isDefault} onChange={(e) => setDraft({ ...draft(), isDefault: e.currentTarget.checked })} />
            {t('settings-sig-default-label')}
          </label>
          <div class={css.actions}>
            <button type="submit" class={css.button} disabled={busy()} data-testid="signature-save">{t('settings-save')}</button>
            <button type="button" class={css.ghost} disabled={busy()} onClick={() => void guard.leave(props.onCancel)}>{t('settings-cancel')}</button>
          </div>
        </fieldset>
      </form>
      <Show when={fieldError()}><p class={css.error} role="alert">{fieldError()}</p></Show>
      {guard.dialog()}
    </section>
  );
}

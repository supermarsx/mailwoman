// Compliance > Policy: the privacy and cookie policy generated from the live
// configuration, with per-section text overrides per language, the operator
// fields it needs, a publish step and the version history with what changed.
// The generator describes configuration; it does not state that a deployment
// meets any law.

import { For, Show, createResource, createSignal, type JSX } from 'solid-js';
import { formatDateTime, t } from '../../../i18n';
import { complianceApi, type ComplianceApi, type Document, type Lang, type PolicySection } from '../../../api/admin/compliance.ts';
import { basePath } from '../../../api/basePath.ts';
import { AdminSubsection } from '../AdminSubsection.tsx';
import { InlineSettingEditor } from '../Settings/InlineSettingEditor.tsx';
import { settingLabel } from '../Settings/labels.ts';
import { describeAdminError } from '../shell/Feedback.tsx';
import * as css from '../admin.css.ts';
import * as ccss from './compliance.css.ts';
import { useSettingRows } from './useSettingRows.ts';

export const POLICY_KEYS = [
  'policy.controller.name', 'policy.controller.address', 'policy.legalBases', 'policy.hosting.provider',
  'policy.hosting.transfers', 'policy.supervisoryAuthority',
] as const;
export const POLICY_OPTION_KEYS = [
  'policy.publicPages', 'policy.autoVersion', 'policy.reconsentOnChange', 'cookies.policyVersion',
  'dsr.contact.name', 'dsr.contact.email', 'dsr.contact.url',
] as const;

const show = (v: unknown): string => (v === null || v === undefined ? '-' : typeof v === 'string' ? v : JSON.stringify(v));

function Override(props: { section: PolicySection; lang: Lang; api: ComplianceApi; canWrite: boolean; onSaved: () => void }): JSX.Element {
  const [draft, setDraft] = createSignal<string | null>(null);
  const [error, setError] = createSignal<unknown>(null);
  const save = async (body: string | null): Promise<void> => {
    try { setError(null); await props.api.saveOverride(props.lang, props.section.id, body); setDraft(null); props.onSaved(); } catch (e) { setError(e); }
  };
  return (
    <div class={ccss.sectionBox} data-testid={`policy-section-${props.section.id}`}>
      <h4 class={css.heading}>{props.section.title}</h4>
      <Show when={props.section.overridden}><span class={css.badge}>{t('compliance-policy-overridden')}</span></Show>
      <Show when={props.section.missing.length > 0}>
        <p role="status">{t('compliance-policy-needs')} {props.section.missing.map((k) => settingLabel(k)).join(', ')}</p>
      </Show>
      <Show when={draft() === null} fallback={
        <>
          <label>
            <span>{t('compliance-policy-override-label')}</span>
            <textarea class={ccss.textarea} value={draft() ?? ''} onInput={(e) => setDraft(e.currentTarget.value)} maxLength={8000} data-testid={`policy-draft-${props.section.id}`} />
          </label>
          <div class={ccss.row}>
            <button type="button" class="btn" onClick={() => void save(draft() ?? '')} data-testid={`policy-save-${props.section.id}`}>{t('compliance-policy-override-save')}</button>
            <button type="button" class="btn btn--ghost" onClick={() => setDraft(null)}>{t('compliance-cancel')}</button>
          </div>
        </>
      }>
        <p class={ccss.body}>{props.section.body}</p>
        <Show when={props.canWrite}>
          <div class={ccss.row}>
            <button type="button" class="btn btn--ghost" onClick={() => setDraft(props.section.overridden ? props.section.body : '')} data-testid={`policy-override-${props.section.id}`}>{t('compliance-policy-override')}</button>
            <Show when={props.section.overridden}>
              <button type="button" class="btn btn--ghost" onClick={() => void save(null)}>{t('compliance-policy-override-remove')}</button>
            </Show>
          </div>
        </Show>
      </Show>
      <Show when={error()}><p role="alert">{describeAdminError(error())}</p></Show>
    </div>
  );
}

export default function PolicyTab(props: { api?: ComplianceApi; canWrite?: boolean }): JSX.Element {
  const api = props.api ?? complianceApi;
  const canWrite = (): boolean => props.canWrite ?? true;
  const [lang, setLang] = createSignal<Lang>('en');
  const [document, setDocument] = createSignal<Document>('privacy');
  const [view, { refetch }] = createResource(() => ({ l: lang(), d: document() }), (k) => api.policy(k.l, k.d));
  const required = useSettingRows(POLICY_KEYS);
  const options = useSettingRows(POLICY_OPTION_KEYS);
  const [message, setMessage] = createSignal<string | null>(null);
  const [failure, setFailure] = createSignal<unknown>(null);
  const publish = async (): Promise<void> => {
    try {
      setFailure(null);
      const r = await api.publish();
      setMessage(r.changed ? t('compliance-policy-published', { version: r.version ?? 0 }) : t('compliance-policy-unchanged'));
      void refetch();
    } catch (e) { setFailure(e); }
  };
  const pageUrl = (format?: string): string => `${basePath()}/${document() === 'cookies' ? 'cookies-policy' : 'privacy'}?lang=${lang()}${format ? `&format=${format}` : ''}`;

  return (
    <div data-testid="compliance-policy" class={css.section}>
      <p class={css.note}>{t('compliance-policy-intro')}</p>
      <Show when={view.error}><p role="alert">{describeAdminError(view.error)}</p></Show>
      <Show when={!view.error ? view.latest : undefined} fallback={<p role="status">{t('admin-shell-loading')}</p>}>
        {(v) => (
          <>
            <AdminSubsection title={t('compliance-policy-status')}>
              <p data-testid="policy-version">{v().version === null ? t('compliance-policy-none') : t('compliance-policy-version', { version: v().version ?? 0, date: v().publishedAt ? formatDateTime(v().publishedAt! * 1000) : '' })}</p>
              <Show when={v().pending.length > 0}>
                <p role="status" data-testid="policy-pending">{t('compliance-policy-pending', { count: v().pending.length })}</p>
                <ul>
                  <For each={v().pending}>{(c) => <li><code>{c.key}</code>: {show(c.old)} {'->'} {show(c.new)}</li>}</For>
                </ul>
              </Show>
              <div class={ccss.row}>
                <Show when={canWrite()}>
                  <button type="button" class="btn" onClick={() => void publish()} data-testid="policy-publish">{t('compliance-policy-publish')}</button>
                </Show>
                <a class="btn btn--ghost" href={pageUrl()} target="_blank" rel="noopener">{t('compliance-policy-open')}</a>
                <a class="btn btn--ghost" href={pageUrl('md')} download="">{t('compliance-policy-md')}</a>
              </div>
              <Show when={message()}><p role="status">{message()}</p></Show>
              <Show when={failure()}><p role="alert">{describeAdminError(failure())}</p></Show>
            </AdminSubsection>

            <AdminSubsection title={t('compliance-policy-required')} description={t('compliance-policy-required-note')}>
              <Show when={v().warnings.length > 0} fallback={<p role="status">{t('compliance-policy-required-ok')}</p>}>
                <ul class={ccss.warn} role="status" data-testid="policy-warnings">
                  <For each={v().warnings}>{(k) => <li>{settingLabel(k)}</li>}</For>
                </ul>
              </Show>
              <For each={required.rows()}>{(row) => <InlineSettingEditor settingKey={row.key} setting={row} onChanged={() => void refetch()} />}</For>
            </AdminSubsection>

            <AdminSubsection title={t('compliance-policy-options')}>
              <For each={options.rows()}>{(row) => <InlineSettingEditor settingKey={row.key} setting={row} onChanged={() => void refetch()} />}</For>
            </AdminSubsection>

            <AdminSubsection title={t('compliance-policy-preview')} description={t('compliance-policy-preview-note')}>
              <div class={ccss.row}>
                <label>{t('compliance-policy-document')}{' '}
                  <select class={ccss.field} value={document()} onChange={(e) => setDocument(e.currentTarget.value as Document)} data-testid="policy-document">
                    <option value="privacy">{t('compliance-policy-doc-privacy')}</option>
                    <option value="cookies">{t('compliance-policy-doc-cookies')}</option>
                  </select>
                </label>
                <label>{t('compliance-policy-language')}{' '}
                  <select class={ccss.field} value={lang()} onChange={(e) => setLang(e.currentTarget.value as Lang)} data-testid="policy-language">
                    <For each={v().langs}>{(l) => <option value={l}>{l}</option>}</For>
                  </select>
                </label>
              </div>
              <For each={v().sections}>{(s) => <Override section={s} lang={lang()} api={api} canWrite={canWrite()} onSaved={() => void refetch()} />}</For>
            </AdminSubsection>

            <AdminSubsection title={t('compliance-policy-history')}>
              <Show when={v().versions.length > 0} fallback={<p>{t('compliance-policy-none')}</p>}>
                <For each={v().versions}>{(ver) => (
                  <details data-testid={`policy-history-${ver.version}`}>
                    <summary>{t('compliance-policy-history-row', { version: ver.version, date: formatDateTime(ver.createdAt * 1000), kind: ver.kind === 'auto' ? t('compliance-policy-kind-auto') : t('compliance-policy-kind-published') })}</summary>
                    <Show when={(ver.changes ?? []).length > 0} fallback={<p>{t('compliance-policy-first')}</p>}>
                      <ul><For each={ver.changes ?? []}>{(c) => <li><code>{c.key}</code>: {show(c.old)} {'->'} {show(c.new)}</li>}</For></ul>
                    </Show>
                  </details>
                )}</For>
              </Show>
            </AdminSubsection>
          </>
        )}
      </Show>
    </div>
  );
}

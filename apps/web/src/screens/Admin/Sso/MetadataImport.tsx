// IdP metadata for a SAML backend: upload the IdP's metadata file (or paste it),
// see what it says, then fill the form from it (t32-cfg §4.12: parsed preview
// instead of a textarea).
//
// The preview lists the IdP's entity id, its sign-in address for the
// HTTP-Redirect binding (the one the server sends the request with), its
// sign-out address and each signing certificate with its end date. "Use these
// values" fills the sign-in and sign-out addresses and the certificate list and
// keeps the metadata text with the backend. A file that is not XML, or has no
// IdP description, is refused with the reason and fills nothing.
//
// Mailwoman does not fetch metadata from an address; the metadata address field
// is stored for reference only (say so instead of pretending to fetch it).

import { For, Show, createSignal, createUniqueId, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { Feedback } from '../shell/Feedback.tsx';
import * as shell from '../shell/shell.css.ts';
import * as ux from '../shell/ux.css.ts';
import * as css from '../shell/extras.css.ts';
import { CertBadge } from './CertList.tsx';
import { certInfo, parseIdpMetadata, readFileText, type IdpMetadata } from './saml.ts';

export interface MetadataImportProps {
  id: string;
  describedBy: string | undefined;
  disabled: boolean;
  /** The metadata text stored with the backend ('' when none). */
  stored: string;
  /** Fill the form: sign-in, sign-out, certificates and the metadata text. */
  onUse: (m: IdpMetadata, xml: string) => void;
  now?: Date | undefined;
}

export function MetadataImport(props: MetadataImportProps): JSX.Element {
  const own = createUniqueId();
  const [text, setText] = createSignal('');
  const [parsed, setParsed] = createSignal<IdpMetadata | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [used, setUsed] = createSignal<string | null>(null);
  let file: HTMLInputElement | undefined;

  function read(xml: string): void {
    setUsed(null);
    setText(xml);
    if (xml.trim() === '') {
      setParsed(null);
      setError(null);
      return;
    }
    const r = parseIdpMetadata(xml);
    if (!r.ok) {
      setParsed(null);
      setError(r.reason === 'not-xml' ? t('admin-sso-meta-not-xml') : t('admin-sso-meta-no-idp'));
      return;
    }
    setError(null);
    setParsed(r.metadata);
  }

  async function fromFile(f: File | undefined): Promise<void> {
    if (!f) return;
    read(await readFileText(f));
    if (file) file.value = '';
  }

  const storedEntity = (): string | null => {
    if (props.stored.trim() === '') return null;
    const r = parseIdpMetadata(props.stored);
    return r.ok ? r.metadata.entityId : null;
  };

  return (
    <div class={css.listEditor} id={props.id} role="group" aria-label={t('admin-sso-meta')} aria-describedby={props.describedBy} data-testid="sso-metadata">
      <p class={shell.help}>
        {props.stored.trim() === ''
          ? t('admin-sso-meta-none-stored')
          : storedEntity() !== null
            ? t('admin-sso-meta-stored', { entity: storedEntity() ?? '' })
            : t('admin-sso-meta-stored-unnamed')}
      </p>
      <div class={shell.field}>
        <label class={shell.fieldLabel} for={`${own}-file`}>
          {t('admin-sso-meta-file')}
        </label>
        <input
          ref={file}
          id={`${own}-file`}
          class={css.fileInput}
          type="file"
          accept=".xml,application/xml,text/xml,application/samlmetadata+xml"
          disabled={props.disabled}
          onChange={(e) => void fromFile(e.currentTarget.files?.[0])}
        />
      </div>
      <div class={shell.field}>
        <label class={shell.fieldLabel} for={`${own}-paste`}>
          {t('admin-sso-meta-paste')}
        </label>
        <textarea
          id={`${own}-paste`}
          class={`${ux.control} ${css.mono}`}
          rows={3}
          spellcheck={false}
          disabled={props.disabled}
          value={text()}
          aria-describedby={error() ? `${own}-error` : undefined}
          aria-invalid={error() ? true : undefined}
          onInput={(e) => read(e.currentTarget.value)}
        />
      </div>
      <Show when={error()}>
        {(msg) => (
          <p id={`${own}-error`} class={shell.fieldError} role="alert">
            <span>{msg()}</span>
          </p>
        )}
      </Show>
      <Show when={parsed()}>
        {(m) => (
          <div class={css.panel} data-testid="sso-metadata-preview">
            <h4 class={css.panelTitle}>{t('admin-sso-meta-preview')}</h4>
            <dl class={css.summary}>
              <dt class={css.summaryTerm}>{t('admin-sso-meta-entity')}</dt>
              <dd class={`${css.summaryValue} ${css.mono}`}>{m().entityId ?? t('admin-sso-meta-missing')}</dd>
              <dt class={css.summaryTerm}>{t('admin-sso-idp-sso-url')}</dt>
              <dd class={`${css.summaryValue} ${css.mono}`}>
                {m().ssoUrl ?? (m().ssoOtherBindingOnly ? t('admin-sso-meta-sso-binding') : t('admin-sso-meta-missing'))}
              </dd>
              <dt class={css.summaryTerm}>{t('admin-sso-idp-slo-url')}</dt>
              <dd class={`${css.summaryValue} ${css.mono}`}>{m().sloUrl ?? t('admin-sso-meta-missing')}</dd>
              <dt class={css.summaryTerm}>{t('admin-sso-idp-certs')}</dt>
              <dd class={css.summaryValue}>
                <Show when={m().certsPem.length > 0} fallback={t('admin-sso-meta-no-certs')}>
                  <ul class={css.listItems}>
                    <For each={m().certsPem.map((p) => certInfo(p))}>
                      {(c) => (
                        <li>
                          <span dir="auto">{c.subject ?? t('admin-sso-cert-no-subject')}</span> <CertBadge cert={c} now={props.now} />
                        </li>
                      )}
                    </For>
                  </ul>
                </Show>
              </dd>
            </dl>
            <div class={css.actions}>
              <button
                type="button"
                class="btn btn--primary"
                disabled={props.disabled || m().ssoUrl === null}
                onClick={() => {
                  props.onUse(m(), text());
                  setUsed(t('admin-sso-meta-used'));
                }}
              >
                {t('admin-sso-meta-use')}
              </button>
            </div>
          </div>
        )}
      </Show>
      <Feedback kind="success" message={used()} />
    </div>
  );
}

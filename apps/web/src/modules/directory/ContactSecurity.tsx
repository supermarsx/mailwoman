// Per-contact security tab (SPEC §13 / §8.2, plan §3 e7): the cert / key / verified
// rows for one address, sourced from the directory. Shows the directory photo, any
// published S/MIME certificates (DER → feeds the existing `mw-crypto` cert path — no
// crypto is done here), and whether the contact has a verified key. EXPORTED for e14
// to mount inside the contact card's Security tab.

import { createResource, For, Show, createMemo, onMount, type JSX } from 'solid-js';
import { DirectoryService, type Fetcher, type DirectoryCert } from './service.ts';
import { t, loadCatalog, isolate } from '../../i18n';
import * as css from './styles.css.ts';

export interface ContactSecurityProps {
  /** The contact's primary address. */
  email: string;
  fetcher?: Fetcher;
  service?: DirectoryService;
}

interface SecurityData {
  certs: DirectoryCert[];
  photoB64: string | null;
}

export function ContactSecurity(props: ContactSecurityProps): JSX.Element {
  onMount(() => void loadCatalog('directory'));
  const service = createMemo(() => props.service ?? new DirectoryService(props.fetcher));
  const [data] = createResource<SecurityData, string>(
    () => props.email,
    async (email): Promise<SecurityData> => {
      const [certs, photoB64] = await Promise.all([service().lookupCert(email), service().lookupPhoto(email)]);
      return { certs, photoB64 };
    },
  );

  return (
    <section class={css.secTab} data-testid="contact-security" aria-label={t('directory-contact-security-label')}>
      <div class={css.secRow}>
        <div style={{ display: 'flex', 'align-items': 'center', gap: '0.75rem' }}>
          <Show when={data()?.photoB64}>
            {(b64) => (
              <img
                class={css.photo}
                src={`data:image/*;base64,${b64()}`}
                alt={t('directory-photo-alt', { email: isolate(props.email) })}
              />
            )}
          </Show>
          <div>
            <p class={css.heading}>{props.email}</p>
            <p class={css.meta}>{t('directory-published-material')}</p>
          </div>
        </div>
      </div>

      <div>
        <p class={css.subHeading}>{t('directory-smime-certificates')}</p>
        <Show
          when={(data()?.certs ?? []).length > 0}
          fallback={<p class={css.meta}>{t('directory-no-cert')}</p>}
        >
          <ul class={css.memberList}>
            <For each={data()?.certs}>
              {(cert) => (
                <li class={css.secRow} data-testid="cert-row">
                  <div>
                    <span class={css.mono}>{cert.fingerprint}</span>
                    <p class={css.meta}>
                      {cert.notAfter
                        ? t('directory-valid-until', { date: cert.notAfter })
                        : t('directory-no-expiry')}
                    </p>
                  </div>
                  <Show
                    when={isCurrent(cert)}
                    fallback={
                      <span class={css.unverified} data-testid="cert-status">
                        <span class={css.statusIcon} aria-hidden="true">
                          ✕
                        </span>
                        {t('directory-cert-expired')}
                      </span>
                    }
                  >
                    <span class={css.verified} data-testid="cert-status">
                      <span class={css.statusIcon} aria-hidden="true">
                        ✓
                      </span>
                      {t('directory-cert-current')}
                    </span>
                  </Show>
                </li>
              )}
            </For>
          </ul>
        </Show>
      </div>

      <Show when={data.loading}>
        <p class={css.meta}>{t('directory-looking-up')}</p>
      </Show>
      <Show when={data.error as unknown}>
        <p class={css.error} role="alert">
          {t('directory-lookup-failed')}
        </p>
      </Show>
    </section>
  );
}

/** Whether a cert's advertised `notAfter` is still in the future (display-only). */
function isCurrent(cert: DirectoryCert): boolean {
  if (cert.notAfter === null) return true;
  const t = Date.parse(cert.notAfter);
  return Number.isNaN(t) || t > Date.now();
}

export default ContactSecurity;

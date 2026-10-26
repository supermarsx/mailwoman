import { createSignal, Show, useContext, type JSX } from 'solid-js';
import { t } from '../../i18n';
import { useAdmin } from './context.ts';
import { ShellContext } from './shell/context.ts';
import { sectionPath } from './shell/router.ts';
import { useDirtyGuard } from './shell/useDirtyGuard.ts';
import { describeAdminError } from './shell/Feedback.tsx';
import { AdminSubsection } from './AdminSubsection.tsx';
import * as css from './admin.css.ts';

export function BanEditor(props: { onSaved?: () => void }): JSX.Element {
  const shell = useContext(ShellContext);
  const { api } = useAdmin();
  const [ip, setIp] = createSignal('');
  const [reason, setReason] = createSignal('');
  const [expiry, setExpiry] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');
  const [saved, setSaved] = createSignal(false);
  useDirtyGuard(() => !saved() && Boolean(ip() || reason() || expiry()));
  const leave = (): void => { void shell?.router.navigate(sectionPath('security', ['bans'])); };
  async function submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (busy()) return;
    const instant = expiry() ? new Date(expiry()) : null;
    if (instant && (!Number.isFinite(instant.getTime()) || instant.getTime() <= Date.now())) {
      setError(t('admin-obs-ban-invalid-expiry')); return;
    }
    setBusy(true); setError('');
    try {
      await api.addBan({ ip: ip().trim(), reason: reason().trim(), expiresAt: instant?.toISOString() ?? null });
      setSaved(true);
      if (props.onSaved) props.onSaved(); else leave();
    } catch (cause) { setError(describeAdminError(cause)); }
    finally { setBusy(false); }
  }
  return <section class={css.section} aria-label={t('admin-obs-ban-add')}>
    <h2 class={css.heading}>{t('admin-obs-ban-add')}</h2>
    <p class={css.note}>{t('admin-obs-bans-note')}</p>
    <Show when={error()}><p role="alert" class={css.error}>{error()}</p></Show>
    <form onSubmit={event => void submit(event)}>
      <fieldset disabled={busy() || (shell !== undefined && !shell.can.can('bans.write'))}>
        <AdminSubsection title={t('admin-obs-ban-add')}>
          <div class={css.grid}>
            <label class="field"><span>{t('admin-obs-ban-ip')}</span><input required maxLength={45} value={ip()} onInput={event => setIp(event.currentTarget.value)} /></label>
            <label class="field"><span>{t('admin-obs-ban-reason')}</span><input maxLength={512} value={reason()} onInput={event => setReason(event.currentTarget.value)} /></label>
            <label class="field"><span>{t('admin-obs-ban-expiry')}</span><input type="datetime-local" value={expiry()} onInput={event => setExpiry(event.currentTarget.value)} /></label>
          </div>
          <p class={css.note}>{t('admin-obs-ban-expiry-help')}</p>
        </AdminSubsection>
        <div class={css.headingRow}><button type="submit" class="btn btn--primary">{t('admin-obs-ban-save')}</button><button type="button" class="btn btn--ghost" onClick={leave}>{t('admin-obs-ban-cancel')}</button></div>
      </fieldset>
    </form>
  </section>;
}

// Mailbox sharing dialog (t13 26.13 E9 mount): the modal host that makes the E8
// ACL editor (`modules/sharing`) reachable from the mailbox context. A "Share
// folder" affordance in the mailbox sidebar opens this for the selected mailbox.
//
// It wires the production ACL client — `createAclClient(accountId, jmap)` over the
// same-origin JMAP transport (`createConfiguredClient().jmap`, the precedent from
// Reader.tsx) — and injects it as `<AclEditor>`'s `client`. The editor self-gates
// its write affordances on the caller's `a` (administer) right (MYRIGHTS), so no
// `canEdit` prop is needed here; the upstream IMAP server is the real enforcer.
//
// Modal mechanics mirror Settings.tsx: a backdrop dialog, a focus trap, Esc/close.

import { createMemo, type JSX } from 'solid-js';
import { t } from '../i18n';
import { createFocusTrap } from '../components/a11y';
import { createConfiguredClient } from '../api/transport.ts';
import { createAclClient } from '../api/acl-types.ts';
import { AclEditor } from '../modules/sharing/index.ts';
import * as css from '../styles/settings.css.ts';

export interface SharingDialogProps {
  /** The mailbox whose ACL is edited. */
  mailboxId: string;
  /** The authenticated account id (drives the JMAP method calls). */
  accountId: string;
  /** Display name for the mailbox (untrusted → bidi-isolated in the editor). */
  mailboxName: string;
  onClose: () => void;
}

export function SharingDialog(props: SharingDialogProps): JSX.Element {
  let panel!: HTMLElement;
  createFocusTrap(() => panel, { onEscape: () => props.onClose() });

  // Build the production ACL client once: same-origin JMAP transport, bound to the
  // account. `client.jmap` is a plain closure (no `this`), so passing it bare is safe.
  const client = createMemo(() => createAclClient(props.accountId, createConfiguredClient().jmap));

  return (
    <div
      class="compose__backdrop"
      role="dialog"
      aria-modal="true"
      aria-label={t('mail-nav-sharing')}
      onClick={(e) => {
        if (e.target === e.currentTarget) props.onClose();
      }}
    >
      <section ref={panel} class={css.panel} tabindex="-1">
        <header class={css.header}>
          <h2>{t('mail-nav-sharing')}</h2>
          <button
            type="button"
            class="btn btn--ghost"
            aria-label={t('common-close')}
            onClick={() => props.onClose()}
          >
            ✕
          </button>
        </header>
        <AclEditor mailboxId={props.mailboxId} mailboxName={props.mailboxName} client={client()} />
      </section>
    </div>
  );
}

export default SharingDialog;

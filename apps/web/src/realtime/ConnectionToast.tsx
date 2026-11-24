import {actionConnectionUnstable} from './connectionStability.ts';
import {backgroundReplayState} from '../sw/register.ts';
import {connectionToastAllowed} from '../api/notificationRuntime.ts';
import {accountHealth} from './accountHealth.ts';
import {primaryMailboxAuthority} from '../api/linkedMailboxSession.ts';
// Connection-status banner (plan §3 e6). Renders a single PERSISTENT banner off
// the realtime controller's connection model: connecting / offline / degraded
// (poll fallback) / auth-expired. These are states, not events, so they stay a
// banner for as long as they hold rather than a toast that times out (t32 f39:
// persistent banners are not toasts). The one transient message, "Reconnected"
// when the socket recovers, is a toast in the shared toast store
// (components/toast). It reads `useRealtime()` so it works with the app
// singleton or a test-provided controller, and offers a Reconnect action for
// the recoverable states.
//
// An ended session (auth-expired) offers "Sign in again" instead: reconnecting
// cannot revive it. The action is the store's `logout()`, which clears the
// session and so returns the shell to the login screen.
//
// It is mounted outside the signed-in branch of App.tsx, so it also renders on
// the login screen. There the model is `idle` (no push transport is wanted) and
// the banner shows nothing: it reports on a session's connection, not on the
// browser. The copy is in the `common` catalog, which ships in the entry chunk.

import { Show, createEffect, useContext, type JSX } from 'solid-js';
import { useRealtime } from './context.ts';
import { toast } from '../components/toast/index.ts';
import { AppContext } from '../state/context.ts';
import type { ConnectionState } from './connection.ts';
import { t } from '../i18n/index.ts';

/** The states that put a banner on screen. */
type BannerState = Exclude<ConnectionState, 'online' | 'idle'> | 'unstable';

// Literal ids at each call, so check-catalog.mjs can resolve them.
const MESSAGES: Record<BannerState, () => string> = {
  unstable: () => t('common-conn-unstable'),
  connecting: () => t('common-conn-connecting'),
  degraded: () => t('common-conn-degraded'),
  offline: () => t('common-conn-offline'),
  'auth-expired': () => t('common-conn-auth-expired'),
};

function isBannerState(s: ConnectionState|'unstable'): s is BannerState {
  return s !== 'online' && s !== 'idle';
}

/** The toast id of "Reconnected", so a second recovery replaces the first. */
const RECONNECTED_TOAST_ID = 'connection-reconnected';

/** These states offer a manual "Reconnect" action from the top of the ladder. */
const RECOVERABLE = new Set<ConnectionState|'unstable'>(['degraded','unstable']);

export function ConnectionToast(): JSX.Element {
  const rt = useRealtime();
  // Read directly rather than through `useApp()`, which throws without a
  // provider: the banner is also rendered on its own against a bare controller.
  const app = useContext(AppContext);

  function signInAgain(): void {
    // The server may refuse the logout request of a session it already ended;
    // `logout()` clears the local session either way, which is the point here.
    void app?.logout('expired').catch(() => undefined);
  }
  const state = rt.connection.state;
  const unstable=()=>rt.connection.unstable()||actionConnectionUnstable();
  const displayedState=():ConnectionState|'unstable'=>{const current=state();return unstable()&&current!=='idle'&&current!=='auth-expired'&&current!=='offline'?'unstable':current;};
  let warned=false;
  createEffect(()=>{
    const active=displayedState()==='unstable';
    if(active&&!warned&&connectionToastAllowed())toast.warning(t('common-conn-unstable'),{id:'connection-unstable'});
    if(!unstable()||state()==='idle'||state()==='auth-expired')toast.dismiss('connection-unstable');
    warned=unstable();
  });
  const previousAccounts=new Map<string,{state:string;unstable:boolean}>();
  createEffect(()=>{
    for(const row of accountHealth()){
      if(row.root!==primaryMailboxAuthority())continue;
      const previous=previousAccounts.get(row.id);previousAccounts.set(row.id,{state:row.state,unstable:row.unstable===true});
      if(previous?.state===row.state&&previous.unstable===(row.unstable===true)||!connectionToastAllowed())continue;
      const id=`account-connection:${row.id}`;
      if(row.unstable&&row.state!=='auth-expired'){if(!previous?.unstable)toast.warning(t('common-conn-account-instability',{account:row.label}),{id});}
      else if(row.state==='online'){if(previous&&(previous.unstable||['offline','degraded','auth-expired'].includes(previous.state)))toast.success(t('common-conn-account-recovered',{account:row.label}),{id});else toast.dismiss(id);}
      else if(row.state==='unknown')toast.dismiss(id);
      else if(row.state!=='syncing')toast.warning(t('common-conn-account-problem',{account:row.label,status:t(`common-conn-account-${row.state}`)}),{id,action:{label:t('common-conn-reconnect'),run:()=>{void app?.switchLinkedMailbox(row.linkId).then(()=>app.refreshCurrentMailbox()).catch(()=>{location.hash='#/settings/linked-accounts';});}}});
    }
  });

  // Whether this session's connection has been down (offline, degraded or
  // auth-expired) since it was last online. A recovery passes through
  // 'connecting' on its way back, so the previous state alone cannot tell a
  // recovery from the first connect of a session (idle → connecting → online).
  let wasDown = false;

  createEffect(() => {
    const s = state();
    if (s === 'online') {
      if (wasDown&&!unstable()&&connectionToastAllowed()) toast.success(t('common-conn-reconnected'), { id: RECONNECTED_TOAST_ID });
      wasDown = false;
    } else if (s === 'idle') {
      // The session ended; there is nothing left to have recovered.
      wasDown = false;
      toast.dismiss(RECONNECTED_TOAST_ID);
    } else {
      // The banner is back: an earlier "Reconnected" is no longer true.
      toast.dismiss(RECONNECTED_TOAST_ID);
      if (s !== 'connecting') wasDown = true;
    }
  });

  return (
    <>
      <Show when={isBannerState(displayedState())?displayedState() as BannerState:undefined} keyed>
        {s => (
            <div
              class={`connection-toast connection-toast--${s}`}
              data-state={s}
              role="status"
              aria-live="polite"
            >
              <span>{s==='offline'&&(app?.offlineQueuePending?.()??0)>0?t('common-conn-offline-queued',{count:app!.offlineQueuePending()}):MESSAGES[s]()}</span>
              <Show when={s==='offline'&&(app?.offlineQueuePending?.()??0)>0&&backgroundReplayState()==='unavailable'}><small>{t('common-conn-background-unavailable')}</small></Show>
              <Show when={RECOVERABLE.has(s)}>
                <button type="button" class="connection-toast__action" onClick={() => rt.reconnect()}>
                  {t('common-conn-reconnect')}
                </button>
              </Show>
              <Show when={s === 'auth-expired' && app !== undefined}>
                <button type="button" class="connection-toast__action" onClick={signInAgain}>
                  {t('common-conn-sign-in-again')}
                </button>
              </Show>
            </div>
          )}
      </Show>
    </>
  );
}

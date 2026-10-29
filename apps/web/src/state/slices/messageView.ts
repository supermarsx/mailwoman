import { createEffect, createRoot, createSignal, on, onCleanup, type Accessor } from 'solid-js';
import type { Email } from '../../api/jmap-types.ts';

/** A change to the open message that this window did not make (t39 L8): it was
 *  moved (to `mailboxIds`) or no longer exists on the server. The reader keeps
 *  showing its copy and says so; it never closes or jumps. */
export type RemoteChange =
  | { readonly kind: 'moved'; readonly mailboxIds: Readonly<Record<string, boolean>> }
  | { readonly kind: 'deleted' };

export interface MessageView {
  openEmail: Accessor<Email | null>;
  sanitizedHtml: Accessor<string | null>;
  readLoading: Accessor<boolean>;
  error: Accessor<unknown>;
  openMessage(id: string): Promise<void>;
  closeMessage(): void;
  setActive(active: boolean): void;
  patch(id: string, patch: Partial<Email>): void;
  /** The server's current `keywords` and `mailboxIds` of the open message
   *  (`null`: it is gone). Keywords follow silently; a different folder set or
   *  a missing message sets `remoteChange`. Local actions call `patch` first,
   *  so only changes made elsewhere get here as differences. */
  reconcile(id: string, current: Pick<Email, 'keywords' | 'mailboxIds'> | null): void;
  remoteChange: Accessor<RemoteChange | null>;
  dismissRemoteChange(): void;
  dispose(): void;
}
export interface MessageViewOptions {
  account: Accessor<string | null>;
  load(account: string, id: string): Promise<{email: Email | null; html: string | null}>;
  readDelay(email: Email): number | null;
  markRead(id: string): void;
}

/** The `true` keys of two JMAP sets are the same. */
function sameSet(a: Readonly<Record<string, boolean>> | undefined, b: Readonly<Record<string, boolean>> | undefined): boolean {
  const on = (set: Readonly<Record<string, boolean>> | undefined): string[] => Object.keys(set ?? {}).filter((key) => set?.[key] === true).sort();
  const left = on(a), right = on(b);
  return left.length === right.length && left.every((key, index) => key === right[index]);
}

/** Every retained reader owns its request fence, body and visible-read timer. */
export function createMessageView(options: MessageViewOptions): MessageView {
  return createRoot(disposeRoot => {
    const [openEmail, setEmail] = createSignal<Email | null>(null);
    const [sanitizedHtml, setHtml] = createSignal<string | null>(null);
    const [readLoading, setLoading] = createSignal(false);
    const [error, setError] = createSignal<unknown>();
    const [remoteChange, setRemoteChange] = createSignal<RemoteChange | null>(null);
    let generation=0;
    let active=false;
    let disposed=false;
    let timer: ReturnType<typeof setTimeout> | undefined;
    // The open message the user marked unread: not marked read again until
    // another message is opened, however often the reader is re-activated.
    let keptUnread: string | null = null;
    function cancelRead(): void {if(timer!==undefined) clearTimeout(timer);timer=undefined;}
    function scheduleRead(): void {
      cancelRead();
      const email=openEmail();
      if(!active || !email || email.keywords?.['$seen'] || email.id===keptUnread) return;
      const delay=options.readDelay(email);
      if(delay===null) return;
      const account=options.account();const request=generation;
      const run=():void => {timer=undefined;if(!disposed && active && request===generation && account===options.account()) options.markRead(email.id);};
      if(delay===0) run();else timer=setTimeout(run,delay);
    }
    function closeMessage(): void {generation++;cancelRead();setEmail(null);setHtml(null);setLoading(false);setError(undefined);setRemoteChange(null);}
    // Close on an account CHANGE only. An effect's first run is deferred when the
    // view is created inside another computation (the workspace restore runs in
    // one), so closing on that first run discarded the load `openMessage` had
    // already started and a restored message tab stayed empty.
    createEffect(on(options.account, closeMessage, {defer:true}));
    onCleanup(cancelRead);
    return {
      openEmail,sanitizedHtml,readLoading,error,
      async openMessage(id) {
        if(disposed) return;
        const account=options.account();if(!account) {closeMessage();return;}
        const request=++generation;cancelRead();setLoading(true);setError(undefined);
        if(keptUnread!==id) keptUnread=null;
        // Retain a body during a same-message refresh, but never show another message's body.
        if(openEmail()?.id!==id) {setEmail(null);setHtml(null);setRemoteChange(null);}
        try {
          const value=await options.load(account,id);
          if(disposed || request!==generation || account!==options.account()) return;
          setEmail(value.email);setHtml(value.html);scheduleRead();
        } catch(reason) {if(!disposed && request===generation && account===options.account()) setError(() => reason);}
        finally {if(!disposed && request===generation) setLoading(false);}
      },
      closeMessage,
      setActive(value) {active=value;scheduleRead();},
      patch(id,patch) {const email=openEmail();if(email?.id===id) {
        if(email.keywords?.['$seen'] && patch.keywords && !patch.keywords['$seen']) keptUnread=id;
        setEmail({...email,...patch});if(patch.keywords?.['$seen']||keptUnread===id) cancelRead();}},
      reconcile(id,current) {
        const email=openEmail();if(disposed || email?.id!==id) return;
        if(current===null) {cancelRead();if(remoteChange()?.kind!=='deleted') setRemoteChange({kind:'deleted'});return;}
        const moved=!sameSet(email.mailboxIds,current.mailboxIds);
        const flagsChanged=!sameSet(email.keywords,current.keywords);
        if(!moved && !flagsChanged) return;
        // Read elsewhere: the pending read here is moot. Unread elsewhere: keep it unread.
        if(flagsChanged && email.keywords?.['$seen'] !== current.keywords?.['$seen']) {cancelRead();if(!current.keywords?.['$seen']) keptUnread=id;}
        setEmail({...email,keywords:{...current.keywords},mailboxIds:{...current.mailboxIds}});
        if(moved) setRemoteChange({kind:'moved',mailboxIds:{...current.mailboxIds}});
      },
      remoteChange,
      dismissRemoteChange() {setRemoteChange(null);},
      dispose() {if(disposed) return;disposed=true;closeMessage();disposeRoot();},
    };
  });
}

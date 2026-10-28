import { createEffect, createRoot, createSignal, on, onCleanup, type Accessor } from 'solid-js';
import type { Email } from '../../api/jmap-types.ts';

export interface MessageView {
  openEmail: Accessor<Email | null>;
  sanitizedHtml: Accessor<string | null>;
  readLoading: Accessor<boolean>;
  error: Accessor<unknown>;
  openMessage(id: string): Promise<void>;
  closeMessage(): void;
  setActive(active: boolean): void;
  patch(id: string, patch: Partial<Email>): void;
  dispose(): void;
}
export interface MessageViewOptions {
  account: Accessor<string | null>;
  load(account: string, id: string): Promise<{email: Email | null; html: string | null}>;
  readDelay(email: Email): number | null;
  markRead(id: string): void;
}

/** Every retained reader owns its request fence, body and visible-read timer. */
export function createMessageView(options: MessageViewOptions): MessageView {
  return createRoot(disposeRoot => {
    const [openEmail, setEmail] = createSignal<Email | null>(null);
    const [sanitizedHtml, setHtml] = createSignal<string | null>(null);
    const [readLoading, setLoading] = createSignal(false);
    const [error, setError] = createSignal<unknown>();
    let generation=0;
    let active=false;
    let disposed=false;
    let timer: ReturnType<typeof setTimeout> | undefined;
    function cancelRead(): void {if(timer!==undefined) clearTimeout(timer);timer=undefined;}
    function scheduleRead(): void {
      cancelRead();
      const email=openEmail();
      if(!active || !email || email.keywords?.['$seen']) return;
      const delay=options.readDelay(email);
      if(delay===null) return;
      const account=options.account();const request=generation;
      const run=():void => {timer=undefined;if(!disposed && active && request===generation && account===options.account()) options.markRead(email.id);};
      if(delay===0) run();else timer=setTimeout(run,delay);
    }
    function closeMessage(): void {generation++;cancelRead();setEmail(null);setHtml(null);setLoading(false);setError(undefined);}
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
        // Retain a body during a same-message refresh, but never show another message's body.
        if(openEmail()?.id!==id) {setEmail(null);setHtml(null);}
        try {
          const value=await options.load(account,id);
          if(disposed || request!==generation || account!==options.account()) return;
          setEmail(value.email);setHtml(value.html);scheduleRead();
        } catch(reason) {if(!disposed && request===generation && account===options.account()) setError(() => reason);}
        finally {if(!disposed && request===generation) setLoading(false);}
      },
      closeMessage,
      setActive(value) {active=value;scheduleRead();},
      patch(id,patch) {const email=openEmail();if(email?.id===id) {setEmail({...email,...patch});if(patch.keywords?.['$seen']) cancelRead();}},
      dispose() {if(disposed) return;disposed=true;closeMessage();disposeRoot();},
    };
  });
}

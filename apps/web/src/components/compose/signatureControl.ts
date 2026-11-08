// The ribbon Signature control's logic, apart from the composer's state. The
// composer gives it a `SignatureHost` (its body, editor and renderers); this
// finds the signature the message holds, applies / replaces / removes it, adds
// a separate copy at the caret, and follows the From identity.

import { createMemo, createSignal } from 'solid-js';
import type { ComposeSignature } from './ComposerExtras.tsx';
import { inertBody } from './richtext.ts';
import {
  applySignatureHtml,
  applySignaturePlain,
  hasPlainSignature,
  locate,
  plainSignatureText,
  removeSignatureHtml,
  removeSignaturePlain,
  type SignaturePosition,
} from './signatureRegion.ts';

export interface SignatureHost {
  /** Capture the account, draft and sending identity before asynchronous rendering. */
  captureAuthority?:()=>()=>boolean;
  richMode(): boolean;
  bodyHtml(): string;
  body(): string;
  /** Replace the whole rich body (one undo step). */
  setHtml(html: string): void;
  setPlain(text: string): void;
  insertAtCursor(html: string, text: string): void;
  /** The signature as sanitised body HTML (images become inline images of the draft). */
  renderHtml(sig: ComposeSignature): Promise<string>;
  renderPlain(sig: ComposeSignature): Promise<string>;
  /** Every selectable signature, across identities. */
  variants(): ComposeSignature[];
  /** The signature an identity gets by default for this message (new or reply). */
  defaultFor(identityId: string): ComposeSignature | null;
  identityId(): string;
  quoteAnchor(): string | undefined;
  quoteKind(): 'reply' | 'tail' | undefined;
  /** The position the settings give a new message. */
  defaultPosition(): SignaturePosition;
  /** Stop the send from adding a signature of its own (`removed`: the user chose none). */
  markInserted(sig: ComposeSignature): void;
  markRemoved(identityId: string): void;
}

export interface AppliedSignature {
  sig: ComposeSignature;
  /** The user changed the text of the region after it was applied. */
  edited: boolean;
}

const squash = (value: string): string => value.replace(/\s+/g, ' ').trim();
const htmlText = (html: string): string => squash(inertBody(html).textContent ?? '');
const sigText = (sig: ComposeSignature): string => (sig.html ? htmlText(sig.html) : squash(sig.plainText?.trim() ? sig.plainText : sig.text));
const plainText = (sig: ComposeSignature): string => squash(sig.plainText?.trim() ? sig.plainText : sig.text);

export function createSignatureController(host: SignatureHost) {
  const [position, setPosition] = createSignal<SignaturePosition>(host.defaultPosition());
  /** What this controller applied last: finds the region again when the editor dropped its marker. */
  const [last, setLast] = createSignal<ComposeSignature | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [offer, setOffer] = createSignal<ComposeSignature | null>(null);

  const candidates = (): ComposeSignature[] => {
    const all = host.variants(), mine = last();
    return mine && !all.some((s) => s.id === mine.id) ? [mine, ...all] : all;
  };

  /** The signature the body holds, if any. */
  const applied = createMemo<AppliedSignature | null>(() => {
    const all = candidates();
    if (host.richMode()) {
      const root = inertBody(host.bodyHtml());
      const marked = locate(root);
      if (marked && marked.id !== null) {
        const sig = all.find((s) => s.id === marked.id);
        if (sig) return { sig, edited: squash(marked.nodes.map((n) => n.textContent ?? '').join(' ')) !== sigText(sig) };
      }
      const preferred = last();
      for (const sig of preferred ? [preferred, ...all] : all) {
        const text = sigText(sig);
        if (text !== '' && locate(root, text)) return { sig, edited: false };
      }
      return null;
    }
    const body = host.body();
    if (!hasPlainSignature(body, host.quoteAnchor())) return null;
    const text = squash(plainSignatureText(body, host.quoteAnchor()) ?? '');
    const mine = last();
    const same = all.find((s) => plainText(s) === text);
    if (same) return { sig: same, edited: false };
    return mine ? { sig: mine, edited: true } : null;
  });

  const opts = () => ({ position: position(), quoteAnchor: host.quoteAnchor(), quoteKind: host.quoteKind(), knownText: last() ? sigText(last()!) : undefined });
  let pending:Promise<unknown>=Promise.resolve();
  const guard = <T,>(run: (assertCurrent:()=>void) => Promise<T>):Promise<T|undefined> => {
    if(busy())return Promise.resolve(undefined);
    setBusy(true);
    const current=host.captureAuthority?.()??(()=>true);
    const assertCurrent=()=>{if(!current())throw new Error('Signature draft authority changed');};
    const work=run(assertCurrent).finally(()=>setBusy(false));
    pending=work;
    return work;
  };

  /** Apply `sig`: replace the region where it stands, else insert by position. */
  async function apply(sig: ComposeSignature): Promise<void> {
    await guard(async assertCurrent => {
      if (host.richMode()) {
        const html = await host.renderHtml(sig);
        assertCurrent();
        host.setHtml(applySignatureHtml(host.bodyHtml(), html, sig.id, { ...opts(), knownText: knownText() }));
      } else {
        const text = await host.renderPlain(sig);
        assertCurrent();
        host.setPlain(applySignaturePlain(host.body(), text, opts()));
      }
      assertCurrent();
      setLast(sig);
      setOffer(null);
      host.markInserted(sig);
    });
  }
  const knownText = (): string | undefined => {
    const a = applied();
    return a ? sigText(a.sig) : last() ? sigText(last()!) : undefined;
  };

  /** Move the applied signature to another position in one edit. */
  async function reposition(next: SignaturePosition): Promise<void> {
    setPosition(next);
    const a = applied();
    if (!a) return;
    await guard(async assertCurrent => {
      const o = { ...opts(), position: next };
      if (host.richMode()) {
        const html = await host.renderHtml(a.sig);
        assertCurrent();
        host.setHtml(applySignatureHtml(removeSignatureHtml(host.bodyHtml(), sigText(a.sig)), html, a.sig.id, { ...o, knownText: undefined }));
      } else {
        const text = plainSignatureText(host.body(), host.quoteAnchor()) ?? (await host.renderPlain(a.sig));
        assertCurrent();
        host.setPlain(applySignaturePlain(removeSignaturePlain(host.body(), host.quoteAnchor()), text, o));
      }
      setLast(a.sig);
    });
  }

  function remove(): void {
    const a = applied();
    if (a) {
      if (host.richMode()) host.setHtml(removeSignatureHtml(host.bodyHtml(), knownText()));
      else host.setPlain(removeSignaturePlain(host.body(), host.quoteAnchor()));
    }
    setLast(null);
    setOffer(null);
    host.markRemoved(a?.sig.identityId ?? host.identityId());
  }

  /** The primary button: remove when one is applied, else insert the identity's default. Returns false when there is none to insert. */
  async function toggle(): Promise<boolean> {
    if (applied()) { remove(); return true; }
    const sig = host.defaultFor(host.identityId());
    if (!sig) return false;
    await apply(sig);
    return true;
  }

  /** A separate copy at the caret: not the managed region, never replaced later. */
  async function insertCopy(sig: ComposeSignature): Promise<void> {
    await guard(async assertCurrent => {
      if (host.richMode()) { const html=await host.renderHtml(sig);assertCurrent();host.insertAtCursor(html,sig.text); }
      else { const text = await host.renderPlain(sig);assertCurrent();host.insertAtCursor('', `-- \n${text}`); }
    });
  }

  /**
   * The From identity changed. An unedited default of the previous identity
   * follows it; an edited one asks (`offer`) and a non-default is left alone.
   */
  async function identityChanged(previousId: string, nextId: string): Promise<void> {
    const a = applied();
    if (!a || previousId === nextId) return;
    const previous = host.defaultFor(previousId);
    if (!previous || previous.id !== a.sig.id) return;
    const next = host.defaultFor(nextId);
    if (next && next.id === a.sig.id) return;
    if (a.edited) { setOffer(next); return; }
    if (next) await apply(next); else remove();
  }

  return {
    applied, position, setPosition, busy, offer,
    whenReady:()=>pending,
    apply, remove, reposition, toggle, insertCopy, identityChanged,
    acceptOffer: async (): Promise<void> => { const o = offer(); setOffer(null); if (o) await apply(o); },
    dismissOffer: (): void => { setOffer(null); },
  };
}

export type SignatureController = ReturnType<typeof createSignatureController>;

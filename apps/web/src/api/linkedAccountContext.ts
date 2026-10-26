import {CAP_MAIL,type JmapSession} from './jmap-types.ts';
import type {Me} from './client.ts';

/** A server-issued link is a selector; the raw account ID is never one. */
export interface LinkedAccountContext {
  linkId:string;
  accountId:string;
  storageAuthority:string;
}

export class LinkedAccountContextError extends Error {
  constructor(readonly code:'accountChanged'|'linkedAccountMismatch') {
    super(code);
    this.name='LinkedAccountContextError';
  }
}

/** Verify both effective endpoints before publishing any child state. */
export function validateLinkedAccountContext(context:LinkedAccountContext,me:Me,session:JmapSession):void {
  if(!context.linkId||!context.accountId||!context.storageAuthority
    ||me.accountId!==context.accountId||me.storageAuthority!==context.storageAuthority
    ||session.primaryAccounts[CAP_MAIL]!==context.accountId
    ||!Object.prototype.hasOwnProperty.call(session.accounts,context.accountId)
    ||me.passwordChangeRequired)throw new LinkedAccountContextError('linkedAccountMismatch');
}

/** Keeps pending selections private until every response has been verified.
 * The caller owns suspension/clearing and supplies context-bound transports.
 * A refused selection never retries as the primary account. */
export function createLinkedAccountSelection(deps:{
  authorize(linkId:string,signal:AbortSignal):Promise<LinkedAccountContext>;
  me(context:LinkedAccountContext,signal:AbortSignal):Promise<Me>;
  session(context:LinkedAccountContext,signal:AbortSignal):Promise<JmapSession>;
  publish(context:LinkedAccountContext,me:Me,session:JmapSession):void;
  isCurrent():boolean;
}) {
  let generation=0;
  let pending:AbortController|undefined;
  function cancel():void {generation++;pending?.abort();pending=undefined;}
  return {
    cancel,
    async select(linkId:string):Promise<void> {
      cancel();
      const request=++generation;
      const controller=new AbortController();
      pending=controller;
      const assertCurrent=():void=>{
        if(controller.signal.aborted||generation!==request||!deps.isCurrent())throw new LinkedAccountContextError('accountChanged');
      };
      try {
        assertCurrent();
        const context=await deps.authorize(linkId,controller.signal);
        assertCurrent();
        if(context.linkId!==linkId)throw new LinkedAccountContextError('linkedAccountMismatch');
        const me=await deps.me(context,controller.signal);
        assertCurrent();
        const session=await deps.session(context,controller.signal);
        assertCurrent();
        validateLinkedAccountContext(context,me,session);
        deps.publish(context,me,session);
      } finally {
        if(pending===controller)pending=undefined;
      }
    },
  };
}

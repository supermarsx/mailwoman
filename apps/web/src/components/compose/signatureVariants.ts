import type {Identity} from '../../api/jmap-types.ts';
import type {ComposeSignature} from './ComposerExtras.tsx';

function signature(identity:Identity,reply:boolean):ComposeSignature|null {
  // Absent fields are the old-server inheritance contract. Explicit null fields
  // represent an intentionally empty reply assignment and must not fall back.
  const resolved=reply&&(identity.replySignatureHtml!==undefined||identity.replySignatureText!==undefined||identity.replySignatureFormat!==undefined);
  const html=(resolved?identity.replySignatureHtml:identity.signatureHtml)??null;
  const text=(resolved?identity.replySignatureText:identity.signatureText)??'';
  const format=(resolved?identity.replySignatureFormat:identity.signatureFormat)??(html?'html':'plain');
  if(!text.trim()&&!html)return null;
  return {id:identity.id,identityId:identity.id,name:identity.name,plainText:text,text:text.trim()?text:(html??'').replace(/<[^>]*>/g,'').trim(),html,format};
}
function same(a:ComposeSignature|null,b:ComposeSignature|null):boolean{return a?.html===b?.html&&a?.text===b?.text&&a?.format===b?.format;}
export function assignedSignature(identity:Identity,reply:boolean):ComposeSignature|null {
  const result=signature(identity,reply);
  return result&&reply&&!same(result,signature(identity,false))?{...result,id:`${identity.id}:reply`}:result;
}
export function signatureVariants(identities:readonly Identity[],labels:{new:string;replies:string}):ComposeSignature[] {
  const variants:ComposeSignature[]=[];
  for(const identity of identities){
    const fresh=assignedSignature(identity,false),reply=assignedSignature(identity,true);
    if(fresh)variants.push({...fresh,name:reply&&reply.id!==fresh.id?`${identity.name} — ${labels.new}`:identity.name});
    if(reply&&reply.id!==fresh?.id)variants.push({...reply,name:`${identity.name} — ${labels.replies}`});
  }
  return variants;
}

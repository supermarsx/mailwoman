// Well-formed responses for synthetic principals, built from the request (method names, tags, ids).
// They pass the Node validators; they are much smaller than real fixture responses.
// ---- well-formed responses built from the request (method names, tags, ids) ----
export function good(name,p,spec,draftId){
 const owner=p.ownerAccountId??p.accountId,acc=p.accountId,ids=p.messageIds;
 const calls=spec.body?JSON.parse(spec.body).methodCalls:[];
 const rows=(...args)=>({methodResponses:calls.map((call,i)=>[call[0],{accountId:acc,...args[i]},call[2]])});
 const page=Math.min(25,ids.length);
 const list=wanted=>({list:wanted.map(id=>({id,subject:'s',keywords:{}})),notFound:[]});
 const upd=id=>({updated:{[id]:null},notUpdated:null});
 const mbox=(id,folder)=>({list:[{id,mailboxIds:{[folder]:true}}],notFound:[]});
 const kw=(id,keyword,on)=>({list:[{id,keywords:on?{[keyword]:true}:{}}],notFound:[]});
 switch(draftId===undefined?name:'draft-discard(id)'){
  case 'authenticated-identity':return {accountId:owner,displayName:'x'};
  case 'Mailbox/get':case 'Email/query':return rows({list:[],ids:[]});
  case 'Email/get':return rows(list(ids.slice(0,2)));
  case 'concurrent-refresh-handler':return {accountId:owner,upstream:true};
  case 'paged-query-and-headers':{const a=ids.slice(0,page),b=ids.slice(page,page+25);return rows({ids:a,total:ids.length,position:0},list(a),{ids:b,total:ids.length,position:page},list(b));}
  case 'body-and-attachment-metadata':{const expected='Representative body sample.'+'x'.repeat((p.bodySampleBytes??27)-27);return rows({list:[{id:ids[0],bodyValues:{'1':{value:expected+'\r\n',isTruncated:false,isEncodingProblem:false},'2':{value:'other'}},attachments:[{blobId:'other',name:'a'},{blobId:p.attachmentBlobId,name:'sample.txt'}]}],notFound:[]});}
  case 'mark-read-and-restore':return rows(upd(ids[0]),kw(ids[0],'$seen',true),upd(ids[0]),kw(ids[0],'$seen',false));
  case 'move-and-restore':return rows(upd(ids[0]),mbox(ids[0],p.archiveId),upd(ids[0]),mbox(ids[0],p.inboxId));
  case 'delete-to-trash-and-restore':return rows(upd(ids[4]),mbox(ids[4],p.trashId),upd(ids[4]),mbox(ids[4],p.inboxId));
  case 'mark-unread-and-restore':return rows(upd(ids[2]),kw(ids[2],'$seen',false),upd(ids[2]),kw(ids[2],'$seen',true));
  case 'flag-and-restore':return rows(upd(ids[1]),kw(ids[1],'$flagged',true),upd(ids[1]),kw(ids[1],'$flagged',false));
  case 'unflag-and-restore':return rows(upd(ids[3]),kw(ids[3],'$flagged',false),upd(ids[3]),kw(ids[3],'$flagged',true));
  case 'jmap-session':return {apiUrl:'/jmap/api',accounts:{[acc]:{name:'x'}},primaryAccounts:{}};
  case 'list-batch':{const a=ids.slice(0,page);return rows({ids:a,queryState:'q7',total:ids.length,position:0},list([...a].reverse()),{ids:[],total:3});}
  case 'changes':return rows({ids:[ids[0]],queryState:'s9'},{oldQueryState:'s9',newQueryState:'s9',removed:[],added:[],total:1},{oldState:'s9',newState:'s9',created:[],updated:[],destroyed:[],hasMoreChanges:false});
  case 'account-preferences':return {version:1,preferences:{theme:'x'}};
  case 'appearance-customization':return {ownerId:owner,document:{version:1},compiledCss:'',lockedFields:[]};
  case 'session-activity':return {ok:true};
  case 'search-planted-term':{const n=Math.min(50,p.plantedQuarterly),q=ids.slice(0,Math.min(n,ids.length));while(q.length<n)q.push('extra'+q.length);return rows({ids:q,total:p.plantedQuarterly},{list:q.map(id=>({id}))});}
  case 'draft-save':case 'draft-discard':return rows({created:{d1:{id:'D-new-1'}},notCreated:null});
  case 'draft-discard(id)':return rows({list:[{id:draftId,mailboxIds:{[p.draftsId]:true},keywords:{$draft:true,$seen:true}}],notFound:[]},{destroyed:[draftId],notDestroyed:null});
  case 'attachment-download':return null;
 }
 throw new Error('no good response for '+name);
}


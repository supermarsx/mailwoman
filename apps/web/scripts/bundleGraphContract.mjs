/** Check a completed Rollup rendered graph, not filename-size heuristics. */
export function mailboxPaneGraphProof(graph,criticalFiles=[]){
 if(graph?.schema!==1||!graph.chunks||typeof graph.chunks!=='object')throw Error('Current rendered bundle graph missing');
 const chunks=graph.chunks;
 const owners=module=>Object.entries(chunks).filter(([,chunk])=>Number(chunk.modules?.[module])>0).map(([name])=>name);
 const closure=(roots,dynamic=false)=>{
  const found=new Set(),queue=[...roots];
  while(queue.length){const name=queue.pop();if(found.has(name))continue;const chunk=chunks[name];if(!chunk)continue;found.add(name);queue.push(...chunk.imports,...(dynamic?chunk.dynamicImports:[]));}
  return found;
 };
 const mailbox=owners('src/screens/Mailbox.tsx');if(mailbox.length!==1)throw Error('Expected one rendered Mailbox root');
 const initial=closure([...mailbox,...Object.keys(chunks).filter(name=>chunks[name].entry),...criticalFiles]),reachable=closure(mailbox,true);
 const deferred={};
 for(const module of ['src/components/Compose.tsx','src/screens/Settings.tsx']){
  const targets=owners(module);if(targets.length!==1||initial.has(targets[0])||!reachable.has(targets[0]))throw Error(`Mailbox heavy UI is missing or statically eager: ${module}`);
  deferred[module]=targets[0];
 }
 return {mailbox:mailbox[0],staticClosure:[...closure(mailbox)].sort(),deferred,sharedModulesAllowed:true};
}

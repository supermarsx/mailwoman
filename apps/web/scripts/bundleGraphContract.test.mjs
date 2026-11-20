import test from 'node:test';
import assert from 'node:assert/strict';
import {mailboxPaneGraphProof} from './bundleGraphContract.mjs';
const chunk=(modules,imports=[],dynamicImports=[],entry=false)=>({modules,imports,dynamicImports,entry});
const valid=()=>({schema:1,chunks:{
 'entry.js':chunk({'src/main.tsx':100},['shared.js'],['mail.js'],true),
 'mail.js':chunk({'src/screens/Mailbox.tsx':100},['shared.js','panes.js']),
 'shared.js':chunk({'src/components/compose/reply.ts':50,'src/screens/Settings/navigation.tsx':50}),
 'panes.js':chunk({'src/screens/LazyMailboxPanes.tsx':50},[],['compose.js','settings.js']),
 'compose.js':chunk({'src/components/Compose.tsx':500},['shared.js']),
 'settings.js':chunk({'src/screens/Settings.tsx':500},['shared.js']),
}});
test('accepts actually deferred roots while allowing shared models and import cycles',()=>{
 const graph=valid();graph.chunks['shared.js'].imports=['mail.js'];
 assert.deepEqual(mailboxPaneGraphProof(graph).deferred,{'src/components/Compose.tsx':'compose.js','src/screens/Settings.tsx':'settings.js'});
});
test('rejects an indirect eager UI edge even if a dynamic edge also exists',()=>{
 const graph=valid();graph.chunks['shared.js'].imports=['compose.js'];assert.throws(()=>mailboxPaneGraphProof(graph),/statically eager/);
});
test('rejects entry-preloaded settings and a missing or orphaned demanded root',()=>{
 assert.throws(()=>mailboxPaneGraphProof(valid(),['settings.js']),/statically eager/);
 let graph=valid();graph.chunks['entry.js'].imports.push('settings.js');assert.throws(()=>mailboxPaneGraphProof(graph),/statically eager/);
 graph=valid();graph.chunks['panes.js'].dynamicImports=['compose.js'];assert.throws(()=>mailboxPaneGraphProof(graph),/missing or statically eager/);
});
test('does not accept a tree-shaken zero-length placeholder as proof of a UI chunk',()=>{
 const graph=valid();graph.chunks['compose.js'].modules['src/components/Compose.tsx']=0;assert.throws(()=>mailboxPaneGraphProof(graph),/missing or statically eager/);
});

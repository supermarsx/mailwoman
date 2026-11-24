// Worker IPC channel with a message inbox and deadline-bound waits (moved out of the controller unchanged).
export function childChannel(child,index){
 const inbox=[],waiters=[];let failed=false,exited=false;
 const fail=()=>{failed=true;for(const waiter of waiters.splice(0))waiter.reject(new Error('Worker exited or failed'));};
 child.on('message',message=>{
  if(message?.type==='failed'){fail();return;}
  const at=waiters.findIndex(waiter=>waiter.type===message?.type&&waiter.stageId===message.stageId);
  if(at<0){if(inbox.length>8)fail();else inbox.push(message);return;}
  waiters.splice(at,1)[0].resolve(message);
 });
 child.once('error',fail);child.once('exit',()=>{exited=true;fail();});
 return {index,send:message=>{if(failed||!child.connected)throw new Error('Worker is unavailable');child.send(message);},wait(type,stageId,timeoutMs){
  if(failed)return Promise.reject(new Error('Worker is unavailable'));
  const at=inbox.findIndex(message=>message?.type===type&&message.stageId===stageId);
  if(at>=0)return Promise.resolve(inbox.splice(at,1)[0]);
  return new Promise((resolve,reject)=>{
   let timer;const waiter={type,stageId,resolve:value=>{clearTimeout(timer);resolve(value);},reject:error=>{clearTimeout(timer);reject(error);}};
   waiters.push(waiter);timer=setTimeout(()=>{const at=waiters.indexOf(waiter);if(at>=0)waiters.splice(at,1);reject(new Error('Worker coordination deadline exceeded'));},timeoutMs);
  });
 },async stop(){
  if(exited)return;
  child.kill('SIGKILL');
  await new Promise((resolve,reject)=>{const timer=setTimeout(()=>reject(new Error('Worker exit unconfirmed')),5000);child.once('exit',()=>{clearTimeout(timer);resolve();});if(exited){clearTimeout(timer);resolve();}});
 }};
}

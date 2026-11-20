// A real, untimed account-boundary check for disposable native fixtures.
// Reports contain counts only; credentials, IDs and returned mail stay private.
export async function verifyFixtureIsolation({base,principals,clients,request=fetch}){
 const failure=()=>new Error('Fixture account isolation was not observed');
 try{
  const target=new URL(base);
  if(target.protocol!=='http:'||target.hostname!=='127.0.0.1'||!target.port||target.pathname!=='/'||target.search||target.hash||target.username||target.password||!Array.isArray(principals)||!Array.isArray(clients)||principals.length!==clients.length||principals.length<1||principals.length>10000)throw failure();
  if(principals.length<2)return {verified:false,status:'insufficient-principals',requests:0};
  const endpoints=[0,principals.length-1];
  for(const index of endpoints){
   const other=index===0?principals.length-1:0,own=principals[index],foreign=principals[other];
   const ownId=own?.messageIds?.[0],foreignId=foreign?.messageIds?.[0];
   if(typeof own?.accountId!=='string'||!own.accountId||own.accountId===foreign?.accountId||typeof ownId!=='string'||!ownId||typeof foreignId!=='string'||!foreignId||ownId===foreignId||!clients[index]?.headers)throw failure();
   const response=await request(new URL('/jmap/api',target),{
    method:'POST',headers:{...clients[index].headers,'content-type':'application/json'},redirect:'error',signal:AbortSignal.timeout(5000),
    body:JSON.stringify({using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[['Email/get',{accountId:own.accountId,ids:[ownId,foreignId,ownId],properties:['id','subject'],mailwomanBodyUse:false},'isolation']]}),
   });
   if(response.status!==200){await response.body?.cancel();throw failure();}
   const reader=response.body?.getReader();if(!reader)throw failure();
   const chunks=[];let size=0;
   try{for(;;){const {done,value}=await reader.read();if(done)break;size+=value.byteLength;if(size>1024*1024){await reader.cancel();throw failure();}chunks.push(Buffer.from(value));}}
   finally{reader.releaseLock();}
   const value=JSON.parse(Buffer.concat(chunks,size).toString('utf8')),rows=value.methodResponses;
   if(!Array.isArray(rows)||rows.length!==1||rows[0]?.[0]!=='Email/get'||rows[0]?.[2]!=='isolation')throw failure();
   const result=rows[0][1];
   if(result?.accountId!==own.accountId||!Array.isArray(result.list)||result.list.length!==2||result.list.some(row=>row?.id!==ownId)||!Array.isArray(result.notFound)||result.notFound.length!==1||result.notFound[0]!==foreignId)throw failure();
  }
  return {verified:true,status:'verified',requests:2,directions:2};
 }catch{throw failure();}
}

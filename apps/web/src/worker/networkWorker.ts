import {createNetworkCore,type NetworkRequest} from './networkCore.ts';
const core=createNetworkCore();
let sequence=0;
const scope=self as unknown as {onconnect:(event:{ports:MessagePort[]})=>void};
scope.onconnect=event=>{for(const port of event.ports){const id=String(++sequence);core.connect(id,reply=>port.postMessage(reply));port.onmessage=message=>{if(message.data?.method==='disconnect')core.disconnect(id);else void core.receive(id,message.data as NetworkRequest);};port.start();}};

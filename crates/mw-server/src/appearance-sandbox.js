// Trusted static bootstrap. Owner code is sent as data to the worker, never run here.
(() => {
  'use strict';
  let connected = false;
  window.addEventListener('message', (event) => {
    if (connected || event.source !== parent || event.data?.kind !== 'appearance-connect' || event.ports.length !== 1) return;
    connected = true;
    const port = event.ports[0];
    let worker = null;
    let timer = null;
    let workerUrl = null;
    let finished = false;
    const finish = (message) => {
      if (finished) return;
      finished = true;
      if (timer !== null) clearTimeout(timer);
      if (worker !== null) worker.terminate();
      if (workerUrl !== null) URL.revokeObjectURL(workerUrl);
      port.postMessage(message);
      port.close();
    };
    port.onmessage = ({ data }) => {
      if (worker !== null || finished || data?.kind !== 'run' || typeof data.code !== 'string' || data.code.length > 16384) return;
      function runner() {
        'use strict';
        const send = self.postMessage.bind(self);
        // Defense in depth. Actual authority boundaries are the opaque origin
        // and inherited network-denying CSP, not these JavaScript replacements.
        for (const name of ['fetch','XMLHttpRequest','WebSocket','WebTransport','EventSource','importScripts','Worker','SharedWorker','indexedDB','caches','BroadcastChannel']) {
          try { Object.defineProperty(self, name, { value: undefined, writable: false, configurable: false }); } catch { /* Browser-defined non-configurable property. */ }
        }
        self.onmessage = ({ data }) => {
          self.onmessage = null;
          const api = Object.freeze({ set(value) { send({kind:'appearance-set', value}); } });
          const context = Object.freeze(data.context);
          try {
            const result = new Function('appearance', 'context', '"use strict";\n' + data.code)(api, context);
            Promise.resolve(result).then(() => send({kind:'done'}), () => send({kind:'error',message:'The appearance script failed.'}));
          } catch { send({kind:'error',message:'The appearance script could not run.'}); }
        };
      }
      try {
        const source = '(' + runner.toString() + ')();';
        workerUrl = URL.createObjectURL(new Blob([source], {type:'text/javascript'}));
        worker = new Worker(workerUrl);
        let messages = 0;
        worker.onmessage = ({ data }) => {
          if (++messages > 32) { finish({kind:'error',message:'The appearance script sent too many updates.'}); return; }
          if (data?.kind === 'done') finish({kind:'done'});
          else if (data?.kind === 'error') finish({kind:'error',message:'The appearance script failed.'});
          else if (data?.kind === 'appearance-set') {
            const value = data.value;
            if (!value || typeof value !== 'object' || Array.isArray(value)) { finish({kind:'error',message:'Appearance updates must be a small object.'}); return; }
            const safe = {}; let keys = 0;
            for (const key in value) {
              if (++keys > 4 || !['accent','density','font','mode'].includes(key) || typeof value[key] !== 'string' || value[key].length > 32) {
                finish({kind:'error',message:'The script requested an unsupported appearance change.'}); return;
              }
              safe[key] = value[key];
            }
            if (keys) port.postMessage({kind:'appearance-set',value:safe});
          }
        };
        worker.onerror = () => finish({kind:'error',message:'The appearance worker could not run.'});
        timer = setTimeout(() => finish({kind:'error',message:'The appearance script exceeded its time limit.'}), 500);
        worker.postMessage({code:data.code, context:data.context});
      } catch { finish({kind:'error',message:'Isolated appearance scripts are unavailable in this browser.'}); }
    };
    port.start();
    port.postMessage({kind:'ready'});
  });
})();

import {render} from 'solid-js/web';
import {Login} from '../src/screens/Login.tsx';
import {AppContext} from '../src/state/context.ts';
import type {AppState} from '../src/state/store.ts';
import '../src/styles/app.css';
const app={login:async()=>undefined,init:async()=>undefined} as unknown as AppState;
render(()=><AppContext.Provider value={app}><Login/></AppContext.Provider>,document.getElementById('root')!);

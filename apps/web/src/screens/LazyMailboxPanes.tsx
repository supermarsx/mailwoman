import type {JSX} from 'solid-js';
import type {SettingsProps} from './Settings.tsx';
import {LazyRoute} from '../components/ErrorBoundary.tsx';
import {loadComposePane,loadSettingsPane} from './mailboxPaneLoaders.ts';
type ComposeProps=Parameters<typeof import('../components/Compose.tsx')['Compose']>[0];
/** Preserve the pane's owner/context and reactive props across the import. A
 * failed import retries through a new LazyRoute attempt, never a cached reject. */
export function LazyCompose(props:ComposeProps):JSX.Element {
 return <LazyRoute name="Mailbox/Compose" load={()=>loadComposePane().then(module=>({default:()=> <module.Compose {...props}/>}))}/>;
}
export function LazySettings(props:SettingsProps):JSX.Element {
 return <LazyRoute name="Mailbox/Settings" load={()=>loadSettingsPane().then(module=>({default:()=> <module.Settings {...props}/>}))}/>;
}

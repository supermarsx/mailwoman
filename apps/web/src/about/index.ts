// About screens: third-party list (t32-e75). See ThirdPartyList.tsx for how to mount it.
//
// `ThirdPartyList` here is lazy: the list (about 450 KB of JSON) and the component
// load as their own chunk when it first renders, inside the screen's <Suspense>.
import { lazy } from 'solid-js';

export const ThirdPartyList = lazy(() => import('./ThirdPartyList.tsx').then((m) => ({ default: m.ThirdPartyList })));
export type { ThirdPartyListProps } from './ThirdPartyList.tsx';
export type { LicenceTextRef, ThirdPartyEntry, ThirdPartyGroup, ThirdPartyKind } from './thirdParty.ts';

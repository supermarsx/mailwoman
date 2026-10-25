import { Show, createUniqueId, type JSX } from 'solid-js';
import * as css from './admin.css.ts';
import * as ux from './shell/ux.css.ts';
/** A named configuration group, with one surface and predictable inner spacing. */
export function AdminSubsection(props:{title:string;description?:string;actions?:JSX.Element;children:JSX.Element}):JSX.Element {
 const heading=createUniqueId();
 return <section class={css.card} aria-labelledby={heading} data-admin-subsection><header class={ux.pageHead}><div class={ux.pageTitleBlock}><h3 id={heading} class={css.heading}>{props.title}</h3><Show when={props.description}><p class={css.note}>{props.description}</p></Show></div><Show when={props.actions}><div class={ux.pageActions}>{props.actions}</div></Show></header><div class={css.subsectionBody}>{props.children}</div></section>;
}

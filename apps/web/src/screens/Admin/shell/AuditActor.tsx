import { Show } from 'solid-js';
import { hrefFor, sectionPath } from './router.ts';
import * as css from './audit.css.ts';

export interface AuditActorFields {
  actor: string;
  actorLabel?: string;
  actorLink?: { section: 'admins' | 'users'; id: string };
}
export function AuditActor(props: AuditActorFields) {
  const label = () => props.actorLabel || props.actor;
  const href = () => {
    const link = props.actorLink;
    return link && (link.section === 'admins' || link.section === 'users') && typeof link.id === 'string' ? hrefFor(sectionPath(link.section, [link.id])) : undefined;
  };
  return <span class={props.actorLabel ? css.actor : css.identifier} title={label()} dir="auto">
    <Show when={href()} fallback={label()}>{path => <a href={path()}>{label()}</a>}</Show>
  </span>;
}

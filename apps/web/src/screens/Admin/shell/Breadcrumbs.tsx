// Breadcrumbs for nested admin screens (t32-ext.md §K; t95).
// The trail itself is the shared `components/breadcrumbs/Breadcrumbs.tsx`
// (`<nav aria-label="Breadcrumb">`, `aria-current="page"` on the last item,
// ellipsis + tooltip on long names, a placeholder while a name loads). Each
// level's label is resolved by `crumbLabels.ts`, never the raw route segment.

import { type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { Breadcrumbs as Trail, type BreadcrumbItem } from '../../../components/breadcrumbs/Breadcrumbs.tsx';
import * as css from './shell.css.ts';

export type Crumb = BreadcrumbItem;

export interface BreadcrumbsProps {
  items: readonly Crumb[];
}

export function Breadcrumbs(props: BreadcrumbsProps): JSX.Element {
  return <Trail class={css.crumbs} items={props.items} label={t('admin-shell-breadcrumb')} loadingLabel={t('admin-crumb-loading')} testId="admin-breadcrumbs" />;
}

import {createResource, Show, type JSX} from 'solid-js';
import {brandFor, brandForHost, loadBrandPath, type BrandCategory, type BrandEntry} from './registry.ts';
import * as css from './brandLogo.css.ts';

export type BrandLogoSize = 16 | 20 | 24 | 32;

export interface BrandLogoProps {
  /** Brand id, display name, alias or domain (`gmail.com`). Unknown values get a monogram. */
  brand: string;
  size?: BrandLogoSize;
  /** Accessible name when the logo stands alone. Defaults to the brand name. */
  title?: string;
  /** True when the name is shown next to the logo: hides it from assistive technology. */
  decorative?: boolean;
  /** Restrict resolution to these categories (use for operator-chosen labels); other names get a monogram. */
  categories?: readonly BrandCategory[] | undefined;
  /** Draw in currentColor instead of the brand colour. */
  mono?: boolean | undefined;
}

/** Deterministic hue (0-359) for the monogram tile, from the name. */
export function monogramHue(name: string): number {
  let h = 0;
  for (const ch of name.toLowerCase()) h = (h * 31 + ch.codePointAt(0)!) % 360;
  return h;
}
export function monogramLetter(name: string): string {
  const match = /[\p{L}\p{N}]/u.exec(name.trim());
  return (match ? match[0] : '?').toUpperCase();
}

function Monogram(props: {name: string}): JSX.Element {
  return (
    <>
      <rect x="1" y="1" width="22" height="22" rx="5" fill={`hsl(${monogramHue(props.name)} 42% 34%)`} />
      <text x="12" y="16.5" text-anchor="middle" font-size="13" font-weight="700" font-family="system-ui, sans-serif" fill="#fff">{monogramLetter(props.name)}</text>
    </>
  );
}

/**
 * A provider or app logo drawn from bundled path data (no network). Single-colour
 * marks follow the text colour unless the brand colour is readable on light and dark
 * surfaces; `mono` forces the text colour. Unknown brands render a themed monogram tile.
 */
export function BrandLogo(props: BrandLogoProps): JSX.Element {
  const entry = (): BrandEntry | undefined => brandFor(props.brand, props.categories);
  const [path] = createResource(() => (entry()?.noMark ? undefined : entry()?.id), id => loadBrandPath(id));
  const label = (): string => props.title ?? entry()?.name ?? props.brand;
  const fill = (): string => (props.mono ? 'currentColor' : entry()?.color ?? 'currentColor');
  return (
    <svg
      class={css.logo} data-size={props.size ?? 20} data-brand={entry()?.id ?? 'monogram'}
      viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"
      role={props.decorative ? undefined : 'img'} aria-label={props.decorative ? undefined : label()} aria-hidden={props.decorative ? 'true' : undefined}
    >
      <Show when={entry() && !entry()!.noMark} fallback={<Monogram name={entry()?.name ?? props.brand} />}>
        <Show when={path()}>{d => <path d={d()} fill={fill()} />}</Show>
      </Show>
    </svg>
  );
}

/** Logo + name in one aligned row; the logo is decorative because the name is visible. */
export function BrandLabel(props: {brand: string; categories?: readonly BrandCategory[] | undefined; label?: string; size?: BrandLogoSize; mono?: boolean | undefined}): JSX.Element {
  return (
    <span class={css.label}>
      <BrandLogo brand={props.brand} categories={props.categories} size={props.size ?? 16} decorative mono={props.mono} />
      <span>{props.label ?? brandFor(props.brand, props.categories)?.name ?? props.brand}</span>
    </span>
  );
}

/** A fixed logo column beside arbitrary content (table cells, list rows) so names stay aligned. */
export function BrandCell(props: {brand: string; categories?: readonly BrandCategory[] | undefined; size?: BrandLogoSize; mono?: boolean | undefined; children: JSX.Element}): JSX.Element {
  return (
    <span class={css.cell}>
      <BrandLogo brand={props.brand} categories={props.categories} size={props.size ?? 20} decorative mono={props.mono} />
      <span class={css.cellText}>{props.children}</span>
    </span>
  );
}

/** Like BrandCell, but only when the host belongs to a known brand; other hosts render unchanged. */
export function HostBrandCell(props: {host: string; size?: BrandLogoSize; children: JSX.Element}): JSX.Element {
  return (
    <Show when={brandForHost(props.host)} fallback={props.children}>
      {brand => <BrandCell brand={brand().id} size={props.size ?? 16}>{props.children}</BrandCell>}
    </Show>
  );
}

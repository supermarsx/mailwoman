// Dashboard page template (t32-cfg UX3): a grid of tiles (stat tiles, charts,
// short lists) under the page heading. Tiles reflow to one column on a phone.
// While the data loads, fails or is not available the page shows that state in
// place of the grid (UX11); a tile whose own figure is missing says
// "Not available" itself (StatTile does).

import { For, Show, type JSX } from 'solid-js';
import { AdminPage, PageState, type PageStateKind } from './PageFrame.tsx';
import * as ux from '../ux.css.ts';

export interface DashboardTile {
  id: string;
  /** Spans the full row (a chart, a list). */
  wide?: boolean;
  render: () => JSX.Element;
}

export interface DashboardPageProps {
  title: string;
  description?: string;
  actions?: JSX.Element;
  /** `ready` shows the tiles; any other state is shown instead of them. */
  state: { kind: 'ready' } | PageStateKind;
  tiles: readonly DashboardTile[];
  /** Accessible name of the tile list. */
  tilesLabel: string;
  testId?: string;
}

export function DashboardPage(props: DashboardPageProps): JSX.Element {
  return (
    <AdminPage
      title={props.title}
      {...(props.description ? { description: props.description } : {})}
      actions={props.actions}
      testId={props.testId}
    >
      <Show when={props.state.kind === 'ready'} fallback={<PageState state={props.state as PageStateKind} />}>
        <ul class={ux.tileGrid} aria-label={props.tilesLabel} role="list">
          <For each={props.tiles}>
            {(tile) => (
              <li class={[ux.tileCell, tile.wide ? ux.tileWide : ''].join(' ').trim()} data-tile={tile.id}>
                {tile.render()}
              </li>
            )}
          </For>
        </ul>
      </Show>
    </AdminPage>
  );
}

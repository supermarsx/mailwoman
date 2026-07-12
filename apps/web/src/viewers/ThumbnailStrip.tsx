import { createResource, For, Show, type JSX } from 'solid-js';
import './viewers.css';
import { categoryOf, type TypeCategory } from './attachments.ts';

/** One attachment as the strip needs it (a subset of `AttachmentItem`). */
export interface StripItem {
  blobId: string;
  name: string;
  mime: string;
  size: number;
}

export interface ThumbnailStripProps {
  items: StripItem[];
  selectedBlobId?: string;
  onSelect?: (item: StripItem) => void;
  /** Resolve an image preview object URL; only called for `image/*` items. */
  resolveThumb?: (item: StripItem) => Promise<string>;
}

const ICON: Record<TypeCategory, string> = {
  image: 'IMG',
  pdf: 'PDF',
  audio: 'AUD',
  video: 'VID',
  text: 'TXT',
  other: 'FILE',
};

function Thumb(props: {
  item: StripItem;
  selected: boolean;
  onSelect: ((item: StripItem) => void) | undefined;
  resolveThumb: ((item: StripItem) => Promise<string>) | undefined;
}): JSX.Element {
  const category = (): TypeCategory => categoryOf(props.item.mime);
  const [preview] = createResource(
    () => (category() === 'image' && props.resolveThumb !== undefined ? props.item : undefined),
    (item) => props.resolveThumb!(item),
  );
  return (
    <button
      type="button"
      class="mw-thumb"
      classList={{ 'mw-thumb--active': props.selected }}
      role="option"
      aria-selected={props.selected}
      title={props.item.name}
      onClick={() => props.onSelect?.(props.item)}
    >
      <span class="mw-thumb__preview" data-category={category()}>
        <Show when={preview()} fallback={<span class="mw-thumb__icon">{ICON[category()]}</span>}>
          {(url) => <img class="mw-thumb__img" src={url()} alt="" />}
        </Show>
      </span>
      <span class="mw-thumb__name">{props.item.name}</span>
    </button>
  );
}

/** Horizontal strip of a message's attachments (plan §2.4). */
export function ThumbnailStrip(props: ThumbnailStripProps): JSX.Element {
  return (
    <div class="mw-thumbs" role="listbox" aria-label="Attachments">
      <For each={props.items}>
        {(item) => (
          <Thumb
            item={item}
            selected={item.blobId === props.selectedBlobId}
            onSelect={props.onSelect}
            resolveThumb={props.resolveThumb}
          />
        )}
      </For>
    </div>
  );
}

export default ThumbnailStrip;

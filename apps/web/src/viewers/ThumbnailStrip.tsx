import { createResource, For, Show, type JSX } from 'solid-js';
import './viewers.css';
import { categoryOf, type TypeCategory } from './attachments.ts';
import { createObjectUrlOwner } from './objectUrl.ts';

/** The preview box `.mw-thumb__preview` reserves (viewers.css). Emitted as the
 *  `<img>`'s intrinsic `width`/`height` so the row has its final height before
 *  the blob decodes — `object-fit: contain` keeps the real aspect ratio. */
const THUMB_W = 96;
const THUMB_H = 60;

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
  // One owner per thumb, so a thumb leaving the strip (or the whole strip
  // unmounting) revokes the Blob its preview pinned. A download that lands after
  // the thumb is gone is revoked on arrival by `adopt` (see objectUrl.ts).
  const urls = createObjectUrlOwner();
  const [preview] = createResource(
    () => (category() === 'image' && props.resolveThumb !== undefined ? props.item : undefined),
    async (item) => urls.adopt(await props.resolveThumb!(item)),
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
          {(url) => (
            <img
              class="mw-thumb__img"
              src={url()}
              alt=""
              loading="lazy"
              decoding="async"
              width={THUMB_W}
              height={THUMB_H}
            />
          )}
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

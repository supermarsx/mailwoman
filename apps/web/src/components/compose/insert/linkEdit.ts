// The hand-over from an editor to the link dialog when an existing link is clicked.

export interface LinkSpecEdit {
  caption: string;
  href: string;
  title: string;
  newTab: boolean;
}

export interface LinkEdit extends LinkSpecEdit {
  /** Rewrite the clicked link in place. */
  apply(next: LinkSpecEdit): void;
  /** Remove the link, keeping its text. */
  remove(): void;
}

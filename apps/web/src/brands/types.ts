export type BrandCategory = 'identity' | 'mail' | 'app' | 'storage' | 'cloud' | 'analytics' | 'push' | 'security' | 'ai';

export interface BrandEntry {
  readonly id: string;
  readonly name: string;
  /** Lower-case alternative names (product names, protocols) that resolve to this brand. */
  readonly aliases: readonly string[];
  /** Domain suffixes: `mail.gmail.com` matches `gmail.com`. */
  readonly hosts: readonly string[];
  readonly category: BrandCategory;
  /** Brand colour, present only when it reads on both light and dark surfaces. */
  readonly color?: string;
}

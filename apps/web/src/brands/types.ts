export type BrandCategory = 'identity' | 'mail' | 'app' | 'storage' | 'cloud' | 'analytics' | 'push' | 'security' | 'ai'
  | 'social' | 'shopping' | 'productivity' | 'gaming' | 'media' | 'finance' | 'logistics' | 'travel' | 'utility'
  | 'tech' | 'channel' | 'professional' | 'pharma' | 'news' | 'fashion' | 'food' | 'auto' | 'dating' | 'payments' | 'health' | 'industry' | 'telecom' | 'insurance' | 'eyewear';

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
  /** True when no mark is bundled for this brand (not in the Simple Icons dataset, or removed from it): the monogram tile is drawn. */
  readonly noMark?: true;
}

import { describe, it, expect, beforeEach } from 'vitest';
import { createRoot } from 'solid-js';
import { createTagsSlice, isLabelKeyword, keywordFor, type TagsSlice } from './tags.ts';
import type { SliceContext } from './context.ts';
import type { Client } from '../../api/client.ts';

const ctx: SliceContext = { client: {} as Client, showToast: () => undefined };

function withTags(run: (tags: TagsSlice) => void): void {
  createRoot((dispose) => {
    run(createTagsSlice(ctx));
    dispose();
  });
}

describe('keywordFor', () => {
  it('normalizes a label name into a keyword token', () => {
    expect(keywordFor('  Work Stuff! ')).toBe('work-stuff');
    expect(keywordFor('$$Junk')).toBe('junk');
    expect(keywordFor('---')).toBe('');
  });
});

describe('isLabelKeyword', () => {
  it('rejects system $-flags, accepts user labels', () => {
    expect(isLabelKeyword('$seen')).toBe(false);
    expect(isLabelKeyword('$flagged')).toBe(false);
    expect(isLabelKeyword('work')).toBe(true);
  });
});

describe('tags registry', () => {
  beforeEach(() => localStorage.clear());

  it('seeds defaults with colors', () => {
    withTags((tags) => {
      expect(tags.tags().length).toBeGreaterThan(0);
      const work = tags.tagByKeyword('work');
      expect(work?.color).toMatch(/^#/);
    });
  });

  it('adds a new label and looks it up by keyword', () => {
    withTags((tags) => {
      const id = tags.addTag('Travel', '#0ea5e9', '✈️');
      expect(id).toBe('travel');
      expect(tags.tagByKeyword('travel')).toMatchObject({ name: 'Travel', color: '#0ea5e9' });
    });
  });

  it('updates an existing label color', () => {
    withTags((tags) => {
      tags.updateTag('work', { color: '#000000' });
      expect(tags.tagByKeyword('work')?.color).toBe('#000000');
    });
  });

  it('deleteTag removes a label from the registry', () => {
    withTags((tags) => {
      tags.deleteTag('work');
      expect(tags.tagByKeyword('work')).toBeUndefined();
    });
  });

  it('persists the registry to localStorage', () => {
    withTags((tags) => tags.addTag('Travel', '#0ea5e9'));
    // A fresh slice rehydrates from storage.
    withTags((tags) => expect(tags.tagByKeyword('travel')).toBeDefined());
  });
});

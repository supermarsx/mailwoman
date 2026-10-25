// Test helpers for the admin UX seam (not a suite: vitest collects only *.test.*).

import axe from 'axe-core';
import { expect } from 'vitest';
import { AdminProblemError } from './adminApi.ts';

/** axe on `node`, colour checks off (jsdom has no layout; e76 runs them in a browser). */
export async function expectNoAxeViolations(node: Element): Promise<void> {
  const result = await axe.run(node, { rules: { 'color-contrast': { enabled: false }, region: { enabled: false } } });
  expect(result.violations.map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(' ')).join(', ')}`)).toEqual([]);
}

/** A server refusal as `adminRequest` would raise it. */
export function problem(status: number, body: Record<string, unknown> = {}, etag: string | null = null): AdminProblemError {
  return new AdminProblemError(status, body, etag);
}

/** No element under `root` carries a style attribute (CSP `style-src 'self'`). */
export function expectNoInlineStyle(root: ParentNode): void {
  const styled = [...root.querySelectorAll('[style]')].map((el) => `${el.tagName.toLowerCase()} style="${el.getAttribute('style')}"`);
  expect(styled).toEqual([]);
}

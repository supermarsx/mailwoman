import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

// Source-level guard that the ~1 MB pdfjs stays OFF the login→inbox entry chunk
// (plan §1.7, §23 bundle gate). The dispatcher must reach EVERY viewer through
// `lazy(() => import(...))` and must NOT statically pull pdfjs; pdfjs may only be
// imported by PdfViewer, which is itself only ever reached lazily — so the
// bundler code-splits it into its own chunk.

function read(rel: string): string {
  return readFileSync(fileURLToPath(new URL(rel, import.meta.url)), 'utf8');
}

describe('viewers are lazy-loaded (code-split off the entry chunk)', () => {
  const dispatcher = read('./AttachmentViewer.tsx');

  it('AttachmentViewer reaches each viewer via lazy(() => import(...))', () => {
    for (const name of ['ImageViewer', 'PdfViewer', 'TextViewer', 'AudioViewer', 'VideoViewer']) {
      const re = new RegExp(`lazy\\(\\s*\\(\\)\\s*=>\\s*import\\(['"]\\./${name}\\.tsx['"]\\)`);
      expect(dispatcher).toMatch(re);
    }
  });

  it('AttachmentViewer does NOT statically import pdfjs or any concrete viewer', () => {
    expect(dispatcher).not.toMatch(/^import[^\n]*pdfjs-dist/m);
    expect(dispatcher).not.toMatch(/^import[^\n]*['"]\.\/PdfViewer\.tsx['"]/m);
    expect(dispatcher).not.toMatch(/^import[^\n]*['"]\.\/ImageViewer\.tsx['"]/m);
  });

  it('pdfjs is imported ONLY by the (lazily-loaded) PdfViewer', () => {
    expect(read('./PdfViewer.tsx')).toMatch(/from 'pdfjs-dist'/);
    for (const other of ['ImageViewer.tsx', 'VideoViewer.tsx', 'AudioViewer.tsx', 'TextViewer.tsx']) {
      expect(read(`./${other}`)).not.toMatch(/pdfjs-dist/);
    }
  });
});

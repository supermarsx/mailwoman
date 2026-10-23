import { existsSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import path from 'node:path';

/**
 * Global teardown of the restyle config: merge this label's per-screen JSON into
 * <T32_OUT>/measure-<label>.json and contrast-<label>.json, and list the
 * screenshots in <T32_OUT>/screens/<label>/INDEX.md. Merges whatever exists on
 * disk for the label, so runs split by project or --grep add up.
 */
function files(dir: string, ext: string): string[] {
  if (!existsSync(dir)) return [];
  return readdirSync(dir).flatMap((name) => {
    const full = path.join(dir, name);
    if (statSync(full).isDirectory()) return files(full, ext);
    return name.endsWith(ext) ? [full] : [];
  });
}

export default function teardown(): void {
  const out = process.env['T32_OUT'];
  const label = process.env['T32_LABEL'];
  if (!out || !label) return;
  for (const kind of ['measure', 'contrast']) {
    const found = files(path.join(out, kind, label), '.json');
    if (found.length === 0) continue;
    const screens: Record<string, unknown> = {};
    for (const f of found.sort()) {
      const r = JSON.parse(readFileSync(f, 'utf8')) as { viewport: string; theme: string; screen: string };
      screens[`${r.viewport}/${r.theme}/${r.screen}`] = r;
    }
    writeFileSync(
      path.join(out, `${kind}-${label}.json`),
      JSON.stringify({ label, kind, image: process.env['T32_IMAGE'], dist: process.env['T32_DIST'], screens }, null, 1),
    );
  }
  const shotsDir = path.join(out, 'screens', label);
  const shots = files(shotsDir, '.png')
    .map((f) => path.relative(shotsDir, f).split(path.sep).join('/'))
    .sort();
  if (shots.length > 0) {
    const byScreen = new Map<string, string[]>();
    for (const s of shots) {
      const [viewport, theme, file] = s.split('/');
      const key = `${viewport} ${file!.replace(/\.png$/, '')}`;
      byScreen.set(key, [...(byScreen.get(key) ?? []), `[${theme}](${s})`]);
    }
    const lines = [
      `# Screenshots: ${label}`,
      '',
      `Image ${process.env['T32_IMAGE'] ?? '?'}, dist ${process.env['T32_DIST'] ?? '?'}. Full-page PNGs, <viewport>/<theme>/<screen>.png.`,
      '',
      '| viewport | screen | files |',
      '|---|---|---|',
      ...[...byScreen.entries()].map(([k, v]) => {
        const [viewport, screen] = k.split(' ');
        return `| ${viewport} | ${screen} | ${v.join(' · ')} |`;
      }),
      '',
    ];
    writeFileSync(path.join(shotsDir, 'INDEX.md'), lines.join('\n'));
  }
}

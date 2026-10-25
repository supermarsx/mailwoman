// Types for about-bundled-packages.mjs (imported by vite.config.ts).
import type { Plugin } from 'vite';

export function packageDirOf(id: string): string | null;
export function bundledPackages(opts?: { main?: boolean }): Plugin;

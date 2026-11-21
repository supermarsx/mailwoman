/** Build-private attestation; no raw HTML or source is serialized. */
export interface SolidTemplateAttestation {
  status: 'rendered';
  codeSha256: string;
  templates: {sha256: string; styleAttributes: number}[];
}
export interface SolidTemplateViolation {
  file: string;
  module: string;
  styleAttributes: number;
}
export function solidModuleSources(ast: unknown): string[];
export function inspectSolidTemplates(code: string, ast: unknown, resolvedSolidSources?: string[]): SolidTemplateAttestation;
export function inspectStandaloneAsset(code: string, ast: unknown): SolidTemplateAttestation;
export function solidTemplateGraphProof(graph: unknown): {
  modules: number;
  templates: number;
  violations: SolidTemplateViolation[];
};

export function solidTemplateAssetCoverage(graph: unknown, actualFiles: string[]): SolidTemplateViolation[];

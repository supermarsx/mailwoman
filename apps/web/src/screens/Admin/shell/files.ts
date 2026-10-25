// Reading a file the admin picked (a metadata file, a certificate, a manifest).

/** The text of a chosen file (FileReader where `Blob.text` is missing). */
export function readFileText(f: Blob): Promise<string> {
  if (typeof f.text === 'function') return f.text();
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result ?? ''));
    reader.onerror = () => reject(reader.error ?? new Error('file read failed'));
    reader.readAsText(f);
  });
}

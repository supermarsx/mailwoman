/** Reserve both panes and the divider inside the measured workspace, rather
 * than allowing a saved list size to push its neighbour out of the grid. */
export function readingPaneSplit(extent: number, preferred: number, bottom = false): {min:number;max:number;size:number} {
  const available = Math.max(0, Math.floor(Number.isFinite(extent) ? extent : 0) - 6);
  const listMinimum = bottom ? 160 : 240;
  const readerMinimum = 280;
  const scale = Math.min(1, available / (listMinimum + readerMinimum));
  const min = Math.floor(listMinimum * scale);
  const max = Math.max(min, Math.min(1200, available - Math.ceil(readerMinimum * scale)));
  const requested = Number.isFinite(preferred) ? preferred : bottom ? 320 : 340;
  return {min,max,size:Math.round(Math.max(min,Math.min(max,requested)))};
}

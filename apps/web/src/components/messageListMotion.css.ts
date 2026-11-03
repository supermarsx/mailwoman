import {style} from '@vanilla-extract/css';

// Removed messages remain only as short-lived, inert paint. Their absolute
// virtual slots never contribute to list height, focus, or scrolling.
export const departure = style({
  pointerEvents:'none',
  userSelect:'none',
  WebkitUserSelect:'none',
  zIndex:1,
  overflow:'hidden',
});

// Standalone PDF viewer window. ssr is already disabled by the root layout.
// Prerender a static shell so the window has a real asset to load; the document
// id is read at runtime from the window label (viewer:<id>) / the ?doc= query.
export const prerender = true;

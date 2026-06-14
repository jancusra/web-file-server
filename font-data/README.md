# Icon webfont source

Source material for the icon font that the demo page (`src/www`) renders via the
`.icon` / `.i-*` CSS classes. This directory is **not** used by the build or the
server at runtime — it only exists so the font can be regenerated.

- `icons/*.svg` — the individual glyphs (`bus`, `configuration`, `earth`,
  `fingerprint`, `poison`, `presenter`).
- `web-font.sfd` — the [FontForge](https://fontforge.org/) project that maps those
  glyphs to code points.

## Regenerating the font

Open `web-font.sfd` in FontForge and export the web formats, then copy the output
into `src/www/fonts/` (`web-font.eot`, `.svg`, `.ttf`, `.woff`, `.woff2`). Those
files are the ones actually served to the browser.

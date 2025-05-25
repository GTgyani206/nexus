import { defineConfig } from 'vite';

export default defineConfig({
  root: 'www', // Set the root directory for Vite to your www folder
  server: {
    // Optional: configure server port or open browser automatically
    // port: 3000, // Default is 5173
    // open: true,
  },
  build: {
    // Output directory for 'npm run build', relative to project root
    // Vite places assets from `www` into this directory.
    // The WASM pkg files are usually in `../pkg` relative to `www` if `outDir` is `dist` at root.
    // If `root` is `www`, `outDir` is relative to `www`.
    // To place build output in `GTTechnologies/vortex-lang-wasm/dist/`:
    outDir: '../dist', 
    emptyOutDir: true, // Clean the dist folder before building
  },
  // Ensure Vite can find the WASM package.
  // If your `pkg` directory is outside `www` (which it is, at project root),
  // you might need to adjust how assets are handled or ensure paths in JS are correct.
  // For development, `../pkg/vortex_lang_wasm.js` in terminal.js should work if pkg is sibling to www.
});

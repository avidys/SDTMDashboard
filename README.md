# Tauri + Sveltekit as frontend

## Installation

Sveltekit created with static adapter, no precompilation

```bash
npx sv create
# simple, ts, static
pnpm install
pnpm install -D @tauri-apps/cli
pnpm install @tauri-apps/api
pnpx tauri init
#	distDir → build (SvelteKit output)
#	devPath → http://localhost:5173
```

```js
//svelte.config.js
import adapter from '@sveltejs/adapter-static';
export default {
  kit: {
    adapter: adapter(),
    alias: { '@': 'src' },
  }
};
// then pnpm build
```

## Development

```sh
#pnpm dev
pnpm tauri dev
```

## Production

Creates .dmg, .msi, or .app packages under src-tauri/target/release/bundle/.

```bash
#pnpm build
pnpm tauri build
```

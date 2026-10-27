import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins:[svelte()],
  clearScreen:false,
  server:{port:1422,strictPort:true,watch:{ignored:['**/src-tauri/**']}},
  build:{outDir:'dist',emptyOutDir:true,rollupOptions:{input:['index.html','industrial.html']}},
});

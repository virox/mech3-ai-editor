import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
export default defineConfig({plugins:[svelte()],clearScreen:false,server:{port:1421,strictPort:true},build:{outDir:'dist-layout',emptyOutDir:true,rollupOptions:{input:'layout.html'}}});

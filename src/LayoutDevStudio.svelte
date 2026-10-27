<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Studio from './Studio.svelte';
  import AiLayoutEditor from './components/AiLayoutEditor.svelte';
  import { DEFAULT_AI_LAYOUT, cloneAiLayout, validateAiLayout, type AiLayout } from './ai-layout';

  let mode=$state<'layout'|'reader'>(localStorage.getItem('mech-reader-layout-dev:mode')==='reader'?'reader':'layout');let layout=$state<AiLayout>(cloneAiLayout(DEFAULT_AI_LAYOUT));let message=$state('Editing the bundled repository layout.');
  let previewLayout=$derived.by(()=>{try{return validateAiLayout(layout)}catch{return DEFAULT_AI_LAYOUT}});
  async function saveLayout(){try{const path=await invoke<string>('save_ai_layout',{layout:validateAiLayout(layout)});message=`Saved ${path}`}catch(error){message=String(error)}}
  async function reloadLayout(){if(!window.confirm('Discard unsaved layout changes and reload src/ai-layout.json?'))return;try{layout=validateAiLayout(await invoke('load_ai_layout'));message='Reloaded src/ai-layout.json'}catch(error){message=String(error)}}
  function setMode(value:'layout'|'reader'){mode=value;localStorage.setItem('mech-reader-layout-dev:mode',value)}
</script>

<div class="layout-dev-shell"><div class="layout-dev-bar"><div><b>AI Vehicles Layout Developer</b><span>{message}</span></div><nav><button class:active={mode==='layout'} onclick={()=>setMode('layout')}>Layout editor</button><button class:active={mode==='reader'} onclick={()=>setMode('reader')}>Live Reader preview</button></nav></div><div class="layout-dev-content"><div class:hidden-mode={mode!=='layout'} aria-hidden={mode!=='layout'}><AiLayoutEditor {layout} onChange={value=>layout=value} onSave={saveLayout} onReload={reloadLayout}/></div><div class="layout-reader-preview" class:hidden-mode={mode!=='reader'} aria-hidden={mode!=='reader'}><Studio aiLayout={previewLayout}/></div></div></div>

<style>
  .layout-dev-shell{height:100vh;overflow:hidden;background:#0b1013}.layout-dev-bar{height:44px;display:flex;align-items:center;justify-content:space-between;padding:0 12px;border-bottom:1px solid #3c4e55;background:#172228;color:#dce6e9}.layout-dev-bar>div{min-width:0;display:flex;align-items:baseline;gap:12px}.layout-dev-bar b{font-size:12px}.layout-dev-bar span{overflow:hidden;color:#84979e;font-size:10px;text-overflow:ellipsis;white-space:nowrap}.layout-dev-bar nav{display:flex;gap:5px}.layout-dev-bar button{padding:6px 10px;font-size:10px}.layout-dev-bar button.active{border-color:#e16a3b;background:#3a2922}.layout-dev-content,.layout-dev-content>div{height:calc(100vh - 45px)}.layout-dev-content>div{height:100%}.hidden-mode{display:none!important}.layout-reader-preview{height:100%;overflow:hidden}:global(.layout-dev-content .layout-editor-shell){height:100%}:global(.layout-reader-preview main){height:calc(100vh - 145px)}
</style>

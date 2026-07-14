<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import type { DocumentSnapshot, Json, Vehicle } from './types';
  import { CHASSIS, GROUPS, STRATEGIES, VLIST } from './types';

  let doc: DocumentSnapshot | null = null;
  let selected = 0; let tab = 'Vehicle'; let busy = false; let message = 'Open an aiv.zrd, JSON export, or Reader ZBD archive.';
  const tabs = ['Vehicle','Activation','Attack','Retreat','Scan / Jump','Fire','Valves','Raw'];
  const booleanFields = new Set(['params[1]','p2[1]','p2[2]','p2[3]','p6[0]','p6[2]','p6[3]','p8[4]','p8[9]','p8[13]','p8[14]','p8[15]','p9[3]','p12[1]','p12[2]','p13[0]']);
  const booleanVlistFields = new Set([1,3,4,6,8,10,12,14,16]);
  const arr = (x: Json|undefined): Json[] => Array.isArray(x) ? x : [];
  function tags(v: Vehicle) { const out:{tag:string;value:Json;at:number}[]=[]; for(let i=4;i<v.record.length-1;i+=2) if(typeof v.record[i]==='string') out.push({tag:v.record[i] as string,value:v.record[i+1],at:i}); return out; }
  function getTag(v:Vehicle, name:string): Json[]|undefined { const t=tags(v).find(x=>x.tag===name); return t && Array.isArray(t.value) ? t.value : undefined; }
  function ensureTag(v:Vehicle,name:string,len:number):Json[]{ let a=getTag(v,name); if(!a){ a=Array(len).fill(0); v.record.push(name,a); doc={...doc!}; } while(a.length<len)a.push(0); return a; }
  function removeTag(v:Vehicle,name:string){ const t=tags(v).find(x=>x.tag===name); if(t){v.record.splice(t.at,2);doc={...doc!};} }
  function scalar(v:Json):string { return v===null?'':String(v); }
  function parse(s:string, old:Json):Json { if(old===null || typeof old==='string') return s===''?null:s; if(s.trim()==='')return 0; const n=Number(s); return Number.isFinite(n)?n:s; }
  function setBase(v:Vehicle,i:number,s:string){ while(v.record.length<4)v.record.push(0); v.record[i]=parse(s,v.record[i]); doc={...doc!}; }
  function setField(a:Json[],i:number,s:string){a[i]=parse(s,a[i]);doc={...doc!};}
  function setBoolean(a:Json[],i:number,checked:boolean){a[i]=checked?1:0;doc={...doc!};}
  function checked(v:Json):boolean{return v===true || Number(v)!==0;}
  async function load(){ const path=await open({multiple:false,filters:[{name:'MW3 AI data',extensions:['zbd','zrd','json']}]}); if(!path)return; busy=true; try{const loaded=await invoke<DocumentSnapshot>('open_document',{path});doc=loaded;selected=0;message=`Loaded ${loaded.vehicles.length} vehicles`; }catch(e){message=String(e)}finally{busy=false} }
  async function saveAs(){if(!doc)return;const ext=doc.sourceKind==='zbd'?'zbd':doc.sourceKind==='zrd'?'zrd':'json';const path=await save({defaultPath:`aiv-edited.${ext}`,filters:[{name:'Output',extensions:[ext]}]});if(!path)return;busy=true;try{await invoke('save_document',{path,snapshot:doc});message=`Saved ${path}`;}catch(e){message=String(e)}finally{busy=false}}
  function duplicate(){if(!doc)return;const v=structuredClone(doc.vehicles[selected]);v.id=`new-${Date.now()}`;v.name+=' Copy';doc.vehicles.splice(selected+1,0,v);selected++;doc={...doc};}
  function addVehicle(){if(!doc)return;doc.vehicles.push({id:`new-${Date.now()}`,name:'New Vehicle',record:[0,[0,0,0],0,0]});selected=doc.vehicles.length-1;doc={...doc};}
  function del(){if(!doc||!doc.vehicles.length)return;doc.vehicles.splice(selected,1);selected=Math.max(0,selected-1);doc={...doc};}
  function move(d:number){if(!doc)return;const n=selected+d;if(n<0||n>=doc.vehicles.length)return;[doc.vehicles[selected],doc.vehicles[n]]=[doc.vehicles[n],doc.vehicles[selected]];selected=n;doc={...doc};}
  function addVlist(v:Vehicle){ensureTag(v,'vlist',0).push(Array(17).fill(0));doc={...doc!};}
</script>

<header><div class="brand"><span class="mark">M3</span><div><h1>Mech AI Editor</h1><small>Recovered file editor</small></div></div><div class="actions"><button onclick={load}>Open</button><button class="primary" disabled={!doc||busy} onclick={saveAs}>Save As…</button></div></header>
<div class="status"><span class:working={busy}>{busy?'Working…':message}</span>{#if doc}<code>{doc.sourcePath}{doc.archiveEntry?` › ${doc.archiveEntry}`:''}</code>{/if}</div>
<main>
  <aside><div class="aside-title"><b>Vehicles</b><button title="New" onclick={addVehicle}>＋</button></div><div class="vehicle-list">{#each doc?.vehicles ?? [] as v,i (v.id)}<button class:active={i===selected} onclick={()=>selected=i}><span>{v.name}</span><small>#{i+1}</small></button>{/each}</div><div class="list-actions"><button onclick={duplicate} disabled={!doc}>Duplicate</button><button onclick={del} disabled={!doc}>Delete</button><button onclick={()=>move(-1)}>↑</button><button onclick={()=>move(1)}>↓</button></div></aside>
  <section class="workspace">
  {#if doc && doc.vehicles[selected]}
    {@const v=doc.vehicles[selected]}
    <nav>{#each tabs as t}<button class:active={tab===t} onclick={()=>tab=t}>{t}</button>{/each}</nav>
    <div class="page">
    {#if tab==='Vehicle'}
      <div class="card"><h2>Vehicle</h2><div class="grid"><label class="wide"><span class="field-title">Name <small>(name)</small></span><input bind:value={v.name} oninput={()=>doc={...doc!}}></label><label><span class="field-title">Net ID <small>(record[0])</small></span><input value={scalar(v.record[0])} oninput={(e)=>setBase(v,0,e.currentTarget.value)}></label><label><span class="field-title">Yaw <small>(record[2])</small></span><input value={scalar(v.record[2])} oninput={(e)=>setBase(v,2,e.currentTarget.value)}></label><label><span class="field-title">Team / allegiance <small>(record[3])</small></span><input value={scalar(v.record[3])} oninput={(e)=>setBase(v,3,e.currentTarget.value)}></label>{#each ['X','Y','Z'] as axis,i}<label><span class="field-title">Position {axis} <small>(record[1][{i}])</small></span><input value={scalar(arr(v.record[1])[i])} oninput={(e)=>{const p=arr(v.record[1]);while(p.length<3)p.push(0);p[i]=parse(e.currentTarget.value,p[i]);v.record[1]=p;doc={...doc!}}}></label>{/each}</div></div>
      {@render Group(v,'p12',GROUPS.p12.title,GROUPS.p12.description,GROUPS.p12.labels)}
      <div class="card"><h2>Identity</h2>{#each ['pilot','skin'] as key}<label><span class="field-title">{key[0].toUpperCase()+key.slice(1)} <small>({key})</small></span><input value={scalar(getTag(v,key)?.[0] ?? tags(v).find(x=>x.tag===key)?.value ?? '')} oninput={(e)=>{const t=tags(v).find(x=>x.tag===key);if(t)v.record[t.at+1]=e.currentTarget.value;else v.record.push(key,e.currentTarget.value);doc={...doc!}}}></label>{/each}</div>
    {:else if tab==='Valves'}
      <div class="card"><div class="title-row"><div><h2>Conditional valve list</h2><p>All 17 recovered VList fields. Event and valve names are checked against linked files when available.</p></div><button onclick={()=>addVlist(v)}>Add condition</button></div>
      {#if doc.linkedValves.length}<div class="references"><b>Valve symbols in archive</b><div>{#each [...new Set(doc.linkedValves)] as name}<code>{name}</code>{/each}</div></div>{/if}
      {#each getTag(v,'vlist') ?? [] as row,ri}<div class="vrow"><div class="row-head"><b>Condition {ri+1}</b><button onclick={()=>{getTag(v,'vlist')!.splice(ri,1);doc={...doc!}}}>Remove</button></div><div class="grid">{#each VLIST as label,i}{@const r=arr(row)}<label><span class="field-title">{label} <small>(vlist[{ri}][{i}])</small></span>{#if i===0}<select value={scalar(r[i])} onchange={(e)=>{while(r.length<17)r.push(0);setField(r,i,e.currentTarget.value);getTag(v,'vlist')![ri]=r}}><option value="0">0 — Close</option><option value="1">1 — Open</option></select>{:else if booleanVlistFields.has(i)}<span class="boolean-control"><input type="checkbox" checked={checked(r[i])} onchange={(e)=>{while(r.length<17)r.push(0);setBoolean(r,i,e.currentTarget.checked);getTag(v,'vlist')![ri]=r}}><span>{checked(r[i])?'Enabled':'Disabled'}</span></span>{:else}<input value={scalar(r[i])} oninput={(e)=>{while(r.length<17)r.push(0);r[i]=parse(e.currentTarget.value,r[i]);getTag(v,'vlist')![ri]=r;doc={...doc!}}}>{/if}</label>{/each}</div></div>{/each}
      {#if !(getTag(v,'vlist')?.length)}<div class="empty">No VList conditions in this vehicle.</div>{/if}</div>
    {:else if tab==='Raw'}
      <div class="card"><h2>Raw vehicle record</h2><p>Read-only. Unknown tags are retained during Save As.</p><pre>{JSON.stringify(v.record,null,2)}</pre></div>
    {:else}
      {#each Object.entries(GROUPS).filter(([,g])=>g.tab===tab) as [key,g]}{@render Group(v,key,g.title,g.description,g.labels)}{/each}
    {/if}</div>
  {:else}<div class="welcome"><div class="mark large">M3</div><h2>Open AI data to begin</h2><p>Direct `.zrd`, extracted `.json`, and Reader `.zbd` archives are supported.</p><button class="primary" onclick={load}>Choose file…</button></div>{/if}
  </section>
</main>

{#snippet Group(v:Vehicle,key:string,title:string,description:string,labels:string[])}
  {@const value=getTag(v,key)}<div class="card"><div class="title-row"><div><h2>{title}</h2><p>{description}</p></div><label class="switch"><input type="checkbox" checked={!!value} onchange={(e)=>e.currentTarget.checked?ensureTag(v,key,labels.length):removeTag(v,key)}> Present</label></div>{#if value}<div class="grid">{#each labels as label,i}<label><span class="field-title">{label} <small>({key}[{i}])</small></span>{#if key==='p2' && i===0}<select value={scalar(value[i])} onchange={(e)=>setField(value,i,e.currentTarget.value)}>{#each STRATEGIES as strategy,si}<option value={String(si)}>{si} — {strategy}</option>{/each}</select>{:else if key==='p8' && i===10}<select value={scalar(value[i])} onchange={(e)=>setField(value,i,e.currentTarget.value)}><option value="0">0</option><option value="1">1</option></select>{:else if key==='p8' && i===16}<select value={scalar(value[i])} onchange={(e)=>setField(value,i,e.currentTarget.value)}><option value="0">0 — Scan</option><option value="1">1 — Jump</option></select>{:else if key==='p12' && i===0}<select value={scalar(value[i])} onchange={(e)=>setField(value,i,e.currentTarget.value)}>{#each CHASSIS as chassis}<option value={chassis}>{chassis}</option>{/each}</select>{:else if booleanFields.has(`${key}[${i}]`)}<span class="boolean-control"><input type="checkbox" checked={checked(value[i])} onchange={(e)=>setBoolean(value,i,e.currentTarget.checked)}><span>{checked(value[i])?'Enabled':'Disabled'}</span></span>{:else}<input value={scalar(value[i])} oninput={(e)=>setField(value,i,e.currentTarget.value)}>{/if}</label>{/each}</div>{:else}<div class="empty">This optional section is not present for this vehicle.</div>{/if}</div>
{/snippet}

<style>
  select { width:100%; color:#e9eff1; background:#0c1316; border:1px solid #34444b; border-radius:5px; padding:8px 9px; outline:none }
  select:focus { border-color:#d96134; box-shadow:0 0 0 2px #d9613428 }
  .field-title small { color:#718087; font-size:.78em; font-weight:400; letter-spacing:0 }
  .boolean-control { min-height:36px; display:flex; align-items:center; gap:9px; padding:7px 9px; color:#c5d0d4; background:#0c1316; border:1px solid #34444b; border-radius:5px }
  .boolean-control input { width:auto; margin:0; accent-color:#d96134 }
</style>

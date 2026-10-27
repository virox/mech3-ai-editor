<script lang="ts">
  import {invoke} from '@tauri-apps/api/core';
  import {listen,type UnlistenFn} from '@tauri-apps/api/event';
  import {open} from '@tauri-apps/plugin-dialog';
  import {onMount} from 'svelte';

  type Mission={path:string;relativePath:string;campaign:string;name:string;kind:string;vehicleCount:number;hasValves:boolean;networkCount:number;regionCount:number};
  type Library={rootPath:string;missions:Mission[];warnings:string[]};
  type Endpoint={id:string;kind:string;label:string;detail:string;layer:string;archiveRelativePath:string;entryName:string;locator:string;value?:number};
  type ChannelIndex={name:string;consumers:Endpoint[];producers:Endpoint[]};
  type MissionIndex={rootPath:string;selectedMission:string;channels:ChannelIndex[];warnings:string[]};
  type Candidate={pid:number;name:string;path?:string;compatible:boolean;detail:string};
  type Valve={name:string;value:number;changedAt?:number};
  type Status={state:string;pid?:number;build?:string;valveCount:number;writeEnabled:boolean;lastSnapshotMs?:number;message?:string};
  type Snapshot={status:Status;valves:Valve[]};
  type TraceObservation={channel:string;kind:string;address:number;moduleOffset?:number;hitCount:number;lastSeenMs:number};
  type TraceStatus={active:boolean;pid?:number;observationCount:number;getterAddress?:number;message:string};
  type TraceSnapshot={status:TraceStatus;observations:TraceObservation[]};
  type Filter='all'|'on'|'off'|'other';

  let library=$state<Library>();let missionIndex=$state<MissionIndex>();let rootPath=$state('');let selectedMission=$state('');
  let candidates=$state<Candidate[]>([]);let status=$state<Status>({state:'disconnected',valveCount:0,writeEnabled:false});let valves=$state<Valve[]>([]);
  let trace=$state<TraceSnapshot>({status:{active:false,observationCount:0,message:'Native trace is disabled.'},observations:[]});
  let search=$state('');let filter=$state<Filter>('all');let expanded=$state<Set<string>>(new Set());let message=$state('');let busy=$state(false);let autoAttached=$state<number>();
  let messageTimer:ReturnType<typeof setTimeout>|undefined;

  const channelMap=()=>new Map((missionIndex?.channels??[]).map(item=>[item.name,item]));
  const traceMap=()=>{const map=new Map<string,TraceObservation[]>();for(const item of trace.observations)map.set(item.channel,[...(map.get(item.channel)??[]),item]);return map};
  const visible=()=>valves.filter(valve=>{const needle=search.trim().toLowerCase();if(needle&&!valve.name.toLowerCase().includes(needle))return false;return filter==='all'||filter==='on'&&valve.value===1||filter==='off'&&valve.value===0||filter==='other'&&valve.value!==0&&valve.value!==1});
  const count=(kind:Filter)=>kind==='all'?valves.length:valves.filter(v=>kind==='on'?v.value===1:kind==='off'?v.value===0:v.value!==0&&v.value!==1).length;
  const hex=(value:number)=>`0x${value.toString(16).padStart(8,'0').toUpperCase()}`;
  const recentlyChanged=(valve:Valve)=>Boolean(valve.changedAt&&Date.now()-valve.changedAt<1100);
  function show(next:string,duration=4500){message=next;if(messageTimer)clearTimeout(messageTimer);messageTimer=setTimeout(()=>message='',duration)}
  function toggleCard(name:string){const next=new Set(expanded);if(next.has(name))next.delete(name);else next.add(name);expanded=next}

  async function chooseRoot(){const picked=await open({directory:true,multiple:false,title:'Select the MechWarrior 3 root folder'});if(typeof picked!=='string')return;busy=true;try{library=await invoke<Library>('dev_scan_game_root',{rootPath:picked});rootPath=library.rootPath;selectedMission=library.missions[0]?.path??'';missionIndex=undefined;show(`Found ${library.missions.length} Reader missions.`)}catch(error){show(String(error),7000)}finally{busy=false}}
  async function loadMission(){if(!selectedMission)return;busy=true;try{missionIndex=await invoke<MissionIndex>('dev_load_mission',{rootPath,missionPath:selectedMission});expanded=new Set();await refreshLive();show(`Loaded ${missionIndex.channels.length} indexed state channels.`)}catch(error){show(String(error),7000)}finally{busy=false}}
  async function changeMission(){if(trace.status.active)await invoke('trace_stop');if(status.state!=='disconnected')await invoke('switchboard_detach');missionIndex=undefined;valves=[];trace={status:{active:false,observationCount:0,message:'Native trace is disabled.'},observations:[]};expanded=new Set();}
  async function attach(pid:number){try{status=await invoke<Status>('switchboard_attach',{pid});autoAttached=pid;show('Attached in monitor-only mode.')}catch(error){show(String(error),7000)}}
  async function detach(){try{status=await invoke<Status>('switchboard_detach');valves=[];trace=await invoke<TraceSnapshot>('trace_snapshot');autoAttached=undefined;show('Detached and cleared the trace session.')}catch(error){show(String(error),7000)}}
  async function refreshLive(){if(!missionIndex)return;try{const snap=await invoke<Snapshot>('switchboard_snapshot');status=snap.status;valves=snap.valves;candidates=await invoke<Candidate[]>('switchboard_candidates');trace=await invoke<TraceSnapshot>('trace_snapshot');if(status.state==='disconnected'){const compatible=candidates.filter(item=>item.compatible);if(compatible.length===1&&autoAttached!==compatible[0].pid)await attach(compatible[0].pid)}}catch(error){show(String(error),7000)}}
  async function toggleTrace(){if(trace.status.active){await invoke('trace_stop');trace=await invoke<TraceSnapshot>('trace_snapshot');show('Native trace stopped and observations cleared.');return}if(!window.confirm('Enable the experimental debugger-style hardware trace? Mech3.exe will be paused briefly while the debugger attaches.'))return;busy=true;try{await invoke('trace_start');trace=await invoke<TraceSnapshot>('trace_snapshot');show(trace.status.message)}catch(error){show(String(error),8000)}finally{busy=false}}
  async function setWriteEnabled(enabled:boolean){try{status=await invoke<Status>('switchboard_set_write_enabled',{enabled})}catch(error){show(String(error),7000)}}
  async function toggleValve(event:MouseEvent,valve:Valve){event.stopPropagation();if(!status.writeEnabled)return;try{await invoke('switchboard_set_valve',{name:valve.name,value:valve.value===0?1:0,expectedValue:valve.value})}catch(error){show(String(error),7000)}}

  onMount(()=>{let unlisten:UnlistenFn|undefined;void listen<Snapshot>('switchboard:snapshot',event=>{status=event.payload.status;valves=event.payload.valves}).then(stop=>unlisten=stop);const timer=window.setInterval(()=>void refreshLive(),700);void chooseRoot();return()=>{clearInterval(timer);unlisten?.();if(messageTimer)clearTimeout(messageTimer)}});
</script>

<main class="dev-shell">
  <header><div class="brand"><span class="mark dev-mark">D</span><div><small>MECHWARRIOR 3 · NATIVE RESEARCH BUILD</small><h1>Valve Switchboard Dev</h1></div></div>{#if missionIndex}<div class:attached={status.state==='attached'} class="connection"><i></i><div><b>{status.state==='attached'?'Live developer session':'Waiting for Mech3.exe'}</b><span>{status.pid?`PID ${status.pid} · ${status.build}`:'Reader index loaded; no process attached'}</span></div></div>{/if}</header>

  {#if !library}
    <section class="setup"><div class="empty-led"></div><h2>Select the MechWarrior 3 root</h2><p>The developer build loads Reader layers before it attaches or starts tracing.</p><button class="primary" disabled={busy} onclick={chooseRoot}>{busy?'Scanning…':'Choose MW3 root…'}</button>{#if message}<p class="setup-message">{message}</p>{/if}</section>
  {:else if !missionIndex}
    <section class="mission-picker"><div class="page-heading"><div><small>READER WORKSPACE</small><h2>Choose campaign and mission</h2><p>{rootPath}</p></div><button class="secondary" onclick={chooseRoot}>Change root</button></div><label><span>Mission Reader</span><select bind:value={selectedMission}>{#each library.missions as mission}<option value={mission.path}>{mission.campaign} · {mission.name} · {mission.kind}</option>{/each}</select></label>{#if selectedMission}{@const selected=library.missions.find(item=>item.path===selectedMission)}{#if selected}<div class="mission-summary"><b>{selected.relativePath}</b><span>{selected.vehicleCount} vehicles · {selected.networkCount} networks · {selected.regionCount} regions · {selected.hasValves?'valves.zrd':'no valves.zrd'}</span></div>{/if}{/if}<button class="primary" disabled={busy||!selectedMission} onclick={loadMission}>{busy?'Loading Reader layers…':'Load mission and begin'}</button>{#if message}<p class="setup-message">{message}</p>{/if}</section>
  {:else}
    <section class="status-bar dev-status"><div><b>{status.valveCount}</b><span>live channels</span></div><div><b>{missionIndex.channels.length}</b><span>Reader channels</span></div><div><b>{trace.status.observationCount}</b><span>runtime addresses</span></div><div class="status-message">{message||trace.status.message||status.message}</div><button class:trace-active={trace.status.active} class="trace-button" disabled={status.state!=='attached'||busy} onclick={toggleTrace}>{trace.status.active?'Stop native trace':'Enable native trace'}</button>{#if status.state!=='disconnected'}<label class="write-toggle"><input type="checkbox" checked={status.writeEnabled} onchange={event=>setWriteEnabled(event.currentTarget.checked)}/> Direct writes</label><button class="secondary" onclick={detach}>Detach</button>{/if}<button class="secondary" onclick={changeMission}>Change mission</button></section>

    {#if status.state==='disconnected'&&candidates.length!==1}<section class="candidates"><h2>Mech3.exe processes</h2>{#each candidates as candidate}<div class:unsupported={!candidate.compatible} class="candidate"><div><b>{candidate.name} · PID {candidate.pid}</b><span>{candidate.path??candidate.detail}</span><small>{candidate.detail}</small></div><button disabled={!candidate.compatible} onclick={()=>attach(candidate.pid)}>Attach</button></div>{/each}</section>{/if}

    <section class="controls"><input aria-label="Search channels" placeholder="Filter channel names…" bind:value={search}/><div class="filters">{#each [['all','All'],['on','On'],['off','Off'],['other','Other']] as item}<button class:active={filter===item[0]} onclick={()=>filter=item[0] as Filter}>{item[1]} <em>{count(item[0] as Filter)}</em></button>{/each}</div></section>
    {#if visible().length}
      <section class="dev-grid">{#each visible() as valve (valve.name)}{@const indexed=channelMap().get(valve.name)}{@const observations=traceMap().get(valve.name)??[]}<article class:changed={recentlyChanged(valve)} class:expanded={expanded.has(valve.name)} class="dev-card"><button class="card-summary" aria-expanded={expanded.has(valve.name)} onclick={()=>toggleCard(valve.name)}><i class:off={valve.value===0} class:on={valve.value===1} class:other={valve.value!==0&&valve.value!==1}></i><div><h2>{valve.name}</h2><span>{indexed?.consumers.length??0} Reader consumer{(indexed?.consumers.length??0)===1?'':'s'} · {observations.length} runtime address{observations.length===1?'':'es'}</span></div><strong>{valve.value}</strong><em>{expanded.has(valve.name)?'−':'+'}</em></button>{#if status.writeEnabled}<button class="quick-write" onclick={event=>toggleValve(event,valve)}>Set {valve.value===0?'1':'0'}</button>{/if}{#if expanded.has(valve.name)}<div class="card-details"><section><h3>Consumers <span>{indexed?.consumers.length??0}</span></h3>{#if indexed?.consumers.length}{#each indexed.consumers as item (item.id)}<div class="endpoint"><b>{item.label}{item.value!==undefined?` = ${item.value}`:''}</b><span>{item.detail}</span><small>{item.layer} · {item.archiveRelativePath} / {item.entryName} · {item.locator}</small></div>{/each}{:else}<p class="unknown-note">No consumer is identified in the loaded Reader stack. Runtime-only or script consumers may still exist.</p>{/if}</section><section><h3>Writers &amp; producers <span>{indexed?.producers.length??0}</span></h3>{#if indexed?.producers.length}{#each indexed.producers as item (item.id)}<div class="endpoint"><b>{item.label}{item.value!==undefined?` = ${item.value}`:''}</b><span>{item.detail}</span><small>{item.layer} · {item.archiveRelativePath} / {item.entryName} · {item.locator}</small></div>{/each}{:else}<p class="unknown-note">No writer is identified in the loaded Reader stack.</p>{/if}</section><section><h3>Runtime trace <span>{observations.length}</span></h3>{#if observations.length}{#each observations as item (`${item.kind}:${item.address}`)}<div class="endpoint runtime"><b>{item.kind}</b><span>{hex(item.address)}{item.moduleOffset!==undefined?` · Mech3.exe+${item.moduleOffset.toString(16).toUpperCase()}`:''}</span><small>{item.hitCount} hit{item.hitCount===1?'':'s'} · last seen {new Date(item.lastSeenMs).toLocaleTimeString()}</small></div>{/each}{:else}<p class="unknown-note">{trace.status.active?'No native read has been observed for this channel yet.':'Enable native trace to collect caller addresses.'}</p>{/if}</section></div>{/if}</article>{/each}</section>
    {:else}<section class="empty compact"><h2>No matching live channels</h2><p>Attach to a running mission or adjust the filters.</p></section>{/if}
  {/if}
</main>

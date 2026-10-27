<script lang="ts">
  import { invoke, isTauri } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { onMount } from 'svelte';

  type Snapshot = { status: { state:string }; valves:{ name:string; value:number }[] };
  type Lamp = { id:string; name:string; on:boolean; kind:string };
  const demoNames = ['MAIN POWER','AUX POWER','OIL PRESS','WATER TEMP','FUEL LEVEL','GEN READY','INLET VALVE','OUTLET VALVE','PUMP 1','PUMP 2','PUMP 3','BYPASS','COMP 1','COMP 2','COMP 3','TURBINE','GENERATOR','EXCITER','HIGH TEMP','LOW PRESS','OVERSPEED','VIBRATION','OIL LEVEL','ALARM'];
  const demo: Lamp[] = demoNames.map((name,i)=>({id:`demo-${i}`,name,on:[0,3,5,6,8,11,13,16,18,21,23].includes(i),kind:'Preview channel'}));
  let powered = $state(true);
  let snapshot = $state<Snapshot|null>(null);
  let receivedLive = $state(false);
  let connection = $state('PREVIEW');
  // The live protocol does not expose a mission title. A URL title can label a session.
  const title = new URLSearchParams(window.location.search).get('mission')?.trim() || 'MECHWARRIOR 3';
  const lamps = $derived<Lamp[]>(snapshot ? [
    ...snapshot.valves.map(v=>({id:`valve:${v.name}`,name:v.name,on:v.value!==0,kind:`State channel · ${v.value}`})),
  ] : receivedLive ? [] : demo);

  onMount(()=>{
    if(!isTauri()) return;
    let disposed=false, busy=false;
    const stops:UnlistenFn[]=[];
    function accept(next:Snapshot){
      if(disposed)return;
      if(next.status.state!=='disconnected')receivedLive=true;
      connection=next.status.state==='attached'?'LIVE':next.status.state==='waitingForRegistry'?'STANDBY':receivedLive?'OFFLINE':'PREVIEW';
      snapshot=next.status.state==='disconnected'?null:next;
    }
    async function poll(){
      if(busy||disposed)return;
      busy=true;
      try{
        const next=await invoke<Snapshot>('switchboard_snapshot');
        accept(next);
        if(next.status.state==='disconnected'){
          const candidates=await invoke<{pid:number;compatible:boolean}[]>('switchboard_candidates');
          const compatible=candidates.filter(c=>c.compatible);
          if(compatible.length===1&&!disposed)await invoke('switchboard_attach',{pid:compatible[0].pid});
        }
      }catch{if(!disposed){connection=receivedLive?'OFFLINE':'PREVIEW';snapshot=null;}}
      finally{busy=false;}
    }
    void listen<Snapshot>('switchboard:snapshot',e=>accept(e.payload)).then(stop=>disposed?stop():stops.push(stop)).catch(()=>undefined);
    void poll();
    const timer=window.setInterval(()=>void poll(),1000);
    return()=>{disposed=true;window.clearInterval(timer);stops.forEach(stop=>stop());};
  });
</script>

<main class="cabinet" class:powered aria-label="Industrial control panel">
  <div class="panel-grain" aria-hidden="true"></div>
  {#each ['tl','tr','bl','br'] as corner}<i class={`screw ${corner}`} aria-hidden="true"></i>{/each}
  <header>
    <div class="mission-tape"><span>{title}</span><i aria-hidden="true"></i></div>
    <div class="engraving">FIELD CONTROL UNIT <span>MK. III / 029</span></div>
  </header>
  <section class="lamp-grid" aria-label="State channels">
    {#each lamps as lamp,i (lamp.id)}
      <div class="channel" style={`--tilt:${[-1.2,.6,-.4,1,-.7,.3][i%6]}deg`}>
        <div class="bezel" class:lit={powered&&lamp.on} role="img" aria-label={`${lamp.name}: ${powered&&lamp.on?'on':'off'}`} title={`${lamp.kind} · ${lamp.name}`}>
          <div class="lens"><span></span></div>
        </div>
        <div class="channel-tape" title={lamp.name}>{lamp.name}</div>
      </div>
    {/each}
    {#if lamps.length===0}<div class="waiting-tape">{connection==='OFFLINE'?'AWAITING SIGNAL':'AWAITING MISSION'}</div>{/if}
  </section>
  <footer>
    <div class="maker-plate"><b>MW / 3</b><span>STATE CHANNEL RECEIVER<br />{connection} · {lamps.length} CIRCUITS</span></div>
    <div class="power-assembly"><span>ON</span><button class="power-switch" role="switch" aria-checked={powered} aria-label="Panel power" onclick={()=>powered=!powered}><i></i></button><span>OFF</span><b>POWER</b></div>
  </footer>
</main>

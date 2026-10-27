<script lang="ts">
  import type { Json } from '../types';
  import { inspectLancemate, type LancemateState } from '../lancemate';
  import { inspectPackedTeams, legacyAssignment, legacyRelation, MAX_CANONICAL_TEAMS, MAX_TEAM_ID, packTeamIds, type LegacyRelation } from '../team';

  interface TeamReference { name:string; teamIds:number[]; layer:string }
  let { value, lancemate, reference, title='Team membership', description='This stores Team IDs, not Friend / Foe / Other.', sourceLabel='record[3]', onChange, onLancemateChange }: { value:Json|undefined; lancemate:Json|undefined; reference?:TeamReference; title?:string; description?:string; sourceLabel?:string; onChange:(value:number)=>void; onLancemateChange:(state:Exclude<LancemateState,'unknown'>)=>void } = $props();
  const relations:LegacyRelation[]=['Friend','Foe','Other'];
  let info=$derived(inspectPackedTeams(value));
  let lancemateInfo=$derived(inspectLancemate(lancemate));

  function teamLabel(id:number):string { return id===0?'Team 0 · unclassified':`Team ${id}` }
  function addTeam(text:string){const id=Number(text);const info=inspectPackedTeams(value);if(!Number.isInteger(id)||!info.valid||info.teamIds.includes(id))return;const packed=packTeamIds([...info.teamIds,id]);if(packed!==undefined)onChange(packed)}
  function removeTeam(id:number){const info=inspectPackedTeams(value);if(!info.valid||info.teamIds.length<=1)return;const packed=packTeamIds(info.teamIds.filter(team=>team!==id));if(packed!==undefined)onChange(packed)}
  function applyRelation(relation:LegacyRelation){if(!reference)return;const info=inspectPackedTeams(value);const id=legacyAssignment(relation,reference.teamIds);if(id===undefined)return;const lossy=info.valid&&(!info.canonical||info.teamIds.length>1||info.teamIds[0]!==id);if(lossy&&!window.confirm(`The legacy ${relation} button replaces the complete team list with Team ${id}. Continue?`))return;onChange(id)}
  function relationReason(relation:LegacyRelation, subject:number[], controlled:number[]):string {
    const friends=controlled.filter(id=>id!==0);
    const friendHits=subject.filter(id=>friends.includes(id));
    const foeHits=subject.filter(id=>id!==0&&!friends.includes(id));
    if(relation==='Friend')return `Shared team ${friendHits.join(', ')}; Friend takes priority over mixed Foe membership.`;
    if(relation==='Foe')return `Non-zero team ${foeHits.join(', ')} is outside the controlled mech's Friend mask.`;
    return 'Only Team 0 is present; Team 0 belongs to neither the Friend nor the Foe mask.';
  }
</script>

<div class="team-editor">
  <div class="team-heading">
    <div><b>{title} <small>({sourceLabel})</small></b>{#if description}<p>{description}</p>{/if}</div>
    {#if info.valid}<span class="packed-value" title="Unsigned 32-bit value written to aiv.zrd">Packed value <code>{info.raw}</code></span>{/if}
  </div>
  {#if info.valid}
    <div class="team-memberships">
      {#each info.teamIds as id}<span class:neutral={id===0} class="team-chip"><b>{teamLabel(id)}</b><button type="button" aria-label={`Remove Team ${id}`} title="Remove team" disabled={info.teamIds.length===1} onclick={()=>removeTeam(id)}>×</button></span>{/each}
      <select aria-label="Add a Team ID" value="" disabled={info.teamIds.length>=MAX_CANONICAL_TEAMS} onchange={e=>addTeam(e.currentTarget.value)}>
        <option value="">{info.teamIds.length>=MAX_CANONICAL_TEAMS?'Maximum of 6 teams':'＋ Add Team ID…'}</option>
        {#each Array.from({length:MAX_TEAM_ID+1},(_,id)=>id).filter(id=>!info.teamIds.includes(id)) as id}<option value={id}>{teamLabel(id)}</option>{/each}
      </select>
    </div>
    {#if !info.canonical}<div class="team-warning"><b>Non-canonical stored lanes:</b> [{info.lanes.join(', ')}]. The raw value remains untouched until you deliberately add, remove, or assign a team; that edit will save the unique IDs in canonical order.</div>{/if}
    <div class="team-explanation"><span><b>Team 0</b> is the special unclassified team.</span><span><b>Teams 1–31</b> become Friend or Foe only when compared with the controlled mech.</span></div>
    <div class="legacy-preview">
      <div class="team-heading">
        <div><b>Legacy Friend / Foe / Other preview</b>{#if reference}<p>Relative to controlled mech <strong>{reference.name}</strong> ({reference.teamIds.map(id=>`Team ${id}`).join(' + ')}, {reference.layer}).</p>{:else}<p>No <code>player</code> record is loaded, so the old dialog's relative classification cannot be derived.</p>{/if}</div>
      </div>
      {#if reference}
        {@const currentRelation=legacyRelation(info.teamIds,reference.teamIds)}
        <div class="legacy-options">
          {#each relations as relation}{@const assignment=legacyAssignment(relation,reference.teamIds)}<button type="button" class:active={currentRelation===relation} disabled={assignment===undefined} onclick={()=>applyRelation(relation)}><span>{relation}{#if currentRelation===relation}<em>current</em>{/if}</span><small>{assignment===undefined?'No matching Team ID':`Old button replaces list → Team ${assignment}`}</small></button>{/each}
        </div>
        <p class="relation-reason"><b>Why {currentRelation}?</b> {relationReason(currentRelation,info.teamIds,reference.teamIds)}</p>
      {/if}
      <div class="lancemate-setting">
        <div class="team-heading">
          <div><b>Lancemate handling <small>(lancemate[0])</small></b><p>Controls whether this vehicle enters the player's Lancemate command system.</p></div>
          <span class="packed-value">Stored <code>{lancemateInfo.stored}</code></span>
        </div>
        <div class="legacy-options lancemate-options">
          <button type="button" class:active={lancemateInfo.state==='auto'} onclick={()=>onLancemateChange('auto')}><span>Auto{#if lancemateInfo.state==='auto'}<em>current</em>{/if}</span><small>Default · friendly non-player vehicles</small></button>
          <button type="button" class:active={lancemateInfo.state==='on'} onclick={()=>onLancemateChange('on')}><span>On{#if lancemateInfo.state==='on'}<em>current</em>{/if}</span><small>Force the Lancemate flag</small></button>
          <button type="button" class:active={lancemateInfo.state==='off'} onclick={()=>onLancemateChange('off')}><span>Off{#if lancemateInfo.state==='off'}<em>current</em>{/if}</span><small>Exclude from Lancemate commands</small></button>
        </div>
        {#if lancemateInfo.state==='unknown'}
          <div class="team-warning"><b>Unknown Lancemate value.</b> It remains untouched until you select Auto, On, or Off.</div>
        {:else if lancemateInfo.state==='auto' && lancemateInfo.present}
          <div class="team-explanation"><span>Auto is stored explicitly. Clicking Auto replaces it with the native missing-tag default.</span></div>
        {:else if lancemateInfo.state==='on'}
          <div class="team-explanation"><span><b>On does not change teams:</b> a hostile vehicle does not become friendly.</span></div>
        {:else if lancemateInfo.state==='off'}
          <div class="team-explanation"><span>Off overrides automatic friendly-team detection.</span></div>
        {:else}
          <div class="team-explanation"><span>Auto is the native default; no <code>lancemate</code> tag is written.</span></div>
        {/if}
      </div>
    </div>
  {:else}
    <div class="team-warning error"><b>Invalid record[3].</b> {info.error} The editor will not reinterpret it automatically. <button type="button" onclick={()=>onChange(0)}>Reset to Team 0</button></div>
  {/if}
</div>

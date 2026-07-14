import type { Json, ReferenceOccurrence, WorkspaceDocument, WorkspaceSnapshot } from './types';

export function array(value: Json | undefined): Json[] { return Array.isArray(value) ? value : []; }
export function pairList(root: Json): Json[] {
  if (!Array.isArray(root)) return [];
  return root.length === 1 && Array.isArray(root[0]) ? root[0] as Json[] : root;
}
export function scalar(value: Json | undefined): string { return value == null ? '' : String(value); }
export function parseScalar(text: string, old: Json | undefined): Json {
  if (old === null || typeof old === 'string' || old === undefined) return text === '' ? '' : text;
  if (text.trim() === '') return 0;
  const number = Number(text);
  return Number.isFinite(number) ? number : text;
}
export function checked(value: Json | undefined): boolean { return value === true || Number(value) !== 0; }
export function documentOf(workspace: WorkspaceSnapshot, kind: WorkspaceDocument['kind']): WorkspaceDocument | undefined { return workspace.documents.find(d => d.kind === kind && d.layer === 'mission') ?? workspace.documents.find(d => d.kind === kind); }
export function documentsOf(workspace: WorkspaceSnapshot, kinds: WorkspaceDocument['kind'][]): WorkspaceDocument[] { return workspace.documents.filter(d => kinds.includes(d.kind)); }
export function archiveFor(workspace: WorkspaceSnapshot, document: WorkspaceDocument): import('./types').WorkspaceArchive | undefined { return workspace.archives.find(a => a.id === document.archiveId); }
export function isLocked(workspace: WorkspaceSnapshot, document: WorkspaceDocument): boolean { return archiveFor(workspace, document)?.locked ?? true; }
export function referenceNames(workspace: WorkspaceSnapshot, targetKind: string): string[] { return [...new Set(workspace.references.filter(r => r.targetKind === targetKind).map(r => r.name))].sort((a,b) => a.localeCompare(b,undefined,{sensitivity:'base'})); }
export function referencesFor(workspace: WorkspaceSnapshot, name: string): ReferenceOccurrence[] { return workspace.references.filter(r => r.name.localeCompare(name,undefined,{sensitivity:'accent'}) === 0); }
export function tags(record: Json[], start = 4): { tag:string; value:Json; at:number }[] { const out=[]; for(let i=start;i<record.length-1;i+=2) if(typeof record[i] === 'string') out.push({tag:record[i] as string,value:record[i+1],at:i}); return out; }
export function tagArray(record: Json[], name: string, start = 4): Json[] | undefined { const found=tags(record,start).find(x=>x.tag===name); return found && Array.isArray(found.value) ? found.value : undefined; }
export function ensureTag(record: Json[], name: string, length: number, start = 4): Json[] { let value=tagArray(record,name,start); if(!value){value=Array(length).fill(0);record.push(name,value)}while(value.length<length)value.push(0);return value; }
export function removeTag(record: Json[], name: string, start = 4): void { const found=tags(record,start).find(x=>x.tag===name); if(found) record.splice(found.at,2); }

export interface VehicleView { id:string; name:string; record:Json[]; pairIndex:number; }
export function vehicles(document: WorkspaceDocument): VehicleView[] { const root=pairList(document.root);const out:VehicleView[]=[];for(let i=0;i+1<root.length;i+=2){if(typeof root[i]==='string'&&Array.isArray(root[i+1]))out.push({id:`${document.id}:v${i/2}`,name:root[i] as string,record:root[i+1] as Json[],pairIndex:i})}return out; }

export interface ValveActionView { id:string; event:string; record:Json[]; pairIndex:number; }
export function valveActions(document: WorkspaceDocument): ValveActionView[] { const root=pairList(document.root);const out:ValveActionView[]=[];for(let i=0;i+1<root.length;i+=2){if(typeof root[i]==='string'&&Array.isArray(root[i+1]))out.push({id:`${document.id}:a${i/2}`,event:root[i] as string,record:root[i+1] as Json[],pairIndex:i})}return out; }

export interface NetNodeView { id:string; key:string; index:number; record:Json[]; pairIndex:number; spatial:boolean; position:Json[]; links:Json[]; specialEdge?:Json[]; tagStart:number; }
export function netNodes(document: WorkspaceDocument): NetNodeView[] { const root=pairList(document.root);const out:NetNodeView[]=[];for(let i=0;i+1<root.length;i+=2){const key=root[i];const record=root[i+1];if(typeof key!=='string'||!key.startsWith('node_')||!Array.isArray(record))continue;const index=Number(key.slice(5));const spatial=typeof record[0]==='number'&&Array.isArray(record[1]);out.push({id:`${document.id}:${key}`,key,index:Number.isFinite(index)?index:out.length,record,pairIndex:i,spatial,position:spatial?array(record[1]):[],links:spatial?array(record[2]):[],specialEdge:!spatial&&Array.isArray(record[0])?record[0] as Json[]:undefined,tagStart:spatial?3:1})}return out; }

export function documentTitle(document: WorkspaceDocument): string { return document.entryName.replace(/\.zrd$/i,''); }
export function layerLabel(layer:string):string { return ({'mission':'Mission','campaign-base':'Campaign base','global-base':'Global base','attached':'Attached'} as Record<string,string>)[layer] ?? layer; }

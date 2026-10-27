export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };
export interface Vehicle { id: string; name: string; record: Json[]; }
export interface DocumentSnapshot {
  sessionId: number; sourcePath: string; sourceKind: 'json'|'zrd'|'zbd'; archiveEntry?: string;
  vehicles: Vehicle[]; linkedValves: string[]; linkedNets: string[]; warnings: string[];
}
export interface FieldSection { title: string; indices: number[]; }
export const GROUPS: Record<string, { title: string; description: string; labels: string[]; sections: FieldSection[]; tab: string }> = {
  params: { title:'Activation & Engagement', description:'Initial enable gate plus activation, follow-target, and fire-control ranges and timing.', tab:'Activation', labels:['Activation range','Enable Initial','Leave-net distance','Return-to-net distance','Approach range','Minimum range','Maximum acquisition range','Firing duration','Cooldown duration'], sections:[{title:'Activation',indices:[0,1]},{title:'Follow Target Parameters',indices:[2,3]},{title:'Direct Attack Ranges',indices:[4,5]},{title:'Fire Control',indices:[6,7,8]}] },
  p2: { title:'Attack Strategy & Retreat Triggers', description:'Attack movement strategy plus the conditions that cause a retreat.', tab:'Attack', labels:['Movement strategy','Net-distance trigger enabled','Time trigger enabled','Health trigger enabled','Strategy radius / Zig Zag length','Zig Zag target distance','Time threshold','Health threshold'], sections:[{title:'Attack Movement',indices:[0,4,5]},{title:'Retreat Triggers',indices:[1,2,3,6,7]}] },
  p6: { title:'Activation & Post-Retreat Disable', description:'Controls initial activation and whether leaving the AI net disables the behavior temporarily or permanently.', tab:'Retreat', labels:['Disabled','Disable duration','For duration','Permanently'], sections:[{title:'Activation',indices:[0]},{title:'After Retreat Leaves the AI Net',indices:[1,2,3]}] },
  p8: { title:'Combat Behavior', description:'Scan/Jump timing, collision avoidance, attack throttle, and scan-trigger settings.', tab:'Attack', labels:['Minimum delay','Maximum delay','Minimum hold duration','Maximum hold duration','Enabled','Alert distance','Minimum distance','Jump ceiling','Approach throttle','Always active','Unknown range mode','Alpha-strike probability','Attack throttle','Aim','Hit','Time','Parameter mode'], sections:[{title:'Scan / Jump Parameters',indices:[0,1,2,3,7,16]},{title:'Collision Avoidance',indices:[4,5,6]},{title:'Attack Throttle',indices:[8,11,12]},{title:'Activation',indices:[9,10]},{title:'Initial Scan Trigger',indices:[13,14,15]}] },
  p9: { title:'Additional Retreat Parameters', description:'Additional retreat behavior; three original meanings have not yet been recovered.', tab:'Retreat', labels:['Unknown','Unknown','Unknown','Enabled','Health-change threshold'], sections:[{title:'Unresolved Parameters',indices:[0,1,2]},{title:'Health-Change Retreat Trigger',indices:[3,4]}] },
  p10:{ title:'Weapon Accuracy', description:'Accuracy settings used while attacking.', tab:'Fire', labels:['Accuracy (0 = most accurate)','Unknown'], sections:[{title:'Firing Accuracy',indices:[0]},{title:'Unresolved Parameter',indices:[1]}] },
  p11:{ title:'Fire-Control Multiplier', description:'Multiplier applied to the fire-control or targeting-computer behavior.', tab:'Fire', labels:['FCS / TC multiplier'], sections:[{title:'Fire Control',indices:[0]}] },
  p12:{ title:'Vehicle Configuration', description:'Default chassis and mission-start state.', tab:'Vehicle', labels:['Default chassis','Objective-conditional activation','Start shut down'], sections:[{title:'Chassis',indices:[0]},{title:'Mission-Start State',indices:[1,2]}] },
  p13:{ title:'Scan Interruption', description:'Controls whether firing immediately ends an active scan.', tab:'Scan / Jump', labels:['End scan when trigger is pulled'], sections:[{title:'Trigger-Pull Behavior',indices:[0]}] }
};
export const VLIST = [
  'Valve state to set',
  'Condition Ammo Enabled',
  'Ammo Threshold',
  'Condition Fired Upon',
  'Condition Health Enabled',
  'Health Threshold',
  'Condition Heat Enabled',
  'Heat Threshold',
  'Condition Enters',
  'Region Name',
  'Condition Target LOS Enabled',
  'Target LOS Range',
  'Condition Target Distance Enabled',
  'Target Distance Threshold',
  'Condition Active',
  'Valve to set',
  'Condition Exits'
];
export const STRATEGIES = ['Direct','Zig Zag','Circle','Jump','Random Move'];
export const CHASSIS = ['notmech','annihilator','avatar','blackhawk','bushwacker','cauldron','champion','daishi','elemental','firefly','madcat','orion','owens','puma','shadowcat','strider','sunder','supernova','thor','vulture'];

export interface MissionSummary { path:string; relativePath:string; campaign:string; name:string; kind:string; vehicleCount:number; hasValves:boolean; networkCount:number; regionCount:number; }
export interface LibrarySnapshot { rootPath:string; missions:MissionSummary[]; warnings:string[]; }
export interface WorkspaceArchive { id:string; path:string; relativePath:string; layer:string; locked:boolean; }
export interface WorkspaceDocument { id:string; archiveId:string; entryName:string; kind:'aiv'|'valves'|'network'|'region'|'targets'|'objectives'|'raw'; layer:string; root:Json; }
export interface ReferenceOccurrence { name:string; role:string; documentId:string; location:string; layer:string; targetKind:string; }
export interface Diagnostic { severity:'info'|'warning'|'error'; code:string; message:string; documentId?:string; location?:string; }
export interface ValveGraphBinding { documentId:string; archiveRelativePath:string; entryName:string; recordKind:string; locator:string; containerPath:number[]; start:number; width:number; mode:string; }
export interface ValveGraphPortBinding { documentId:string; path:number[]; removeContainerPath?:number[]; removeStart?:number; removeWidth?:number; }
export interface ValveGraphPort { id:string; label:string; direction:'read'|'write'; channel:string; value?:number; binding:ValveGraphPortBinding; }
export interface ValveGraphNode { id:string; kind:string; category:'producer'|'logic'|'consumer'|'unknown'; label:string; subtitle:string; layer:string; locked:boolean; order:number; binding:ValveGraphBinding; ports:ValveGraphPort[]; properties:Record<string,Json>; nativeRecord:Json; }
/** `kind: 'route'` links a network gate to an assignment on a link leaving that node (topology, not a channel). */
export interface ValveGraphEdge { id:string; from:string; to:string; channel:string; kind?:'route'; }
export interface ValveGraphChannel { id:string; name:string; producerCount:number; consumerCount:number; layers:string[]; }
export interface ValveGraphHost { id:string; kind:'valves-document'|'vehicle-vlist'|'target-health'|'network-tags'|'objective-list'; label:string; documentId:string; archiveRelativePath:string; entryName:string; layer:string; locked:boolean; path:number[]; properties:Record<string,Json>; }
export interface ValveGraphDiagnostic { severity:'info'|'warning'|'error'; code:string; message:string; nodeId?:string; documentId?:string; }
export interface ValveGraphSnapshot { scopeKey:string; nodes:ValveGraphNode[]; edges:ValveGraphEdge[]; channels:ValveGraphChannel[]; hosts:ValveGraphHost[]; diagnostics:ValveGraphDiagnostic[]; }
export type ValveGraphEdit =
  | {operation:'addBlock';blockKind:string;hostId:string;nativeRecord:Json}
  | {operation:'updateBlock';nodeId:string;nativeRecord:Json}
  | {operation:'removeBlock';nodeId:string}
  | {operation:'connect';portId:string;channelName:string}
  | {operation:'disconnect';portId:string}
  | {operation:'renameChannel';oldName:string;newName:string;occurrenceIds:string[]};
export interface WorkspaceSnapshot { sessionId:number; rootPath:string; selectedMission:string; archives:WorkspaceArchive[]; documents:WorkspaceDocument[]; references:ReferenceOccurrence[]; diagnostics:Diagnostic[]; valveGraph:ValveGraphSnapshot; }
export interface ExportResult { files:string[]; changedDocuments:number; }

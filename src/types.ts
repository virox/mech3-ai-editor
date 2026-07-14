export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };
export interface Vehicle { id: string; name: string; record: Json[]; }
export interface DocumentSnapshot {
  sessionId: number; sourcePath: string; sourceKind: 'json'|'zrd'|'zbd'; archiveEntry?: string;
  vehicles: Vehicle[]; linkedValves: string[]; linkedNets: string[]; warnings: string[];
}
export const GROUPS: Record<string, { title: string; description: string; labels: string[]; tab: string }> = {
  params: { title:'Activation & Engagement', description:'Activation, follow-target, and fire-control ranges and timing.', tab:'Activation', labels:['Activation range','Initially enabled','Follow target: leave-net distance','Follow target: return-to-net distance','Direct attack: approach range','Direct attack: minimum range','Fire control: maximum acquisition range','Fire control: firing duration','Fire control: cooldown duration'] },
  p2: { title:'Attack Strategy & Retreat Triggers', description:'Attack movement strategy plus the conditions that cause a retreat.', tab:'Attack', labels:['Attack movement strategy','Retreat trigger: net distance enabled','Retreat trigger: time enabled','Retreat trigger: health enabled','Strategy radius / Zig Zag length','Zig Zag target distance','Retreat time threshold','Retreat health threshold'] },
  p6: { title:'Activation & Post-Retreat Disable', description:'Controls initial activation and whether leaving the AI net disables the behavior temporarily or permanently.', tab:'Retreat', labels:['Activation disabled','Post-retreat disable duration','Disable after leaving net: for duration','Disable after leaving net: permanently'] },
  p8: { title:'Combat Behavior', description:'Scan/Jump timing, collision avoidance, attack throttle, and scan-trigger settings.', tab:'Attack', labels:['Scan/Jump: minimum delay','Scan/Jump: maximum delay','Scan/Jump: minimum hold duration','Scan/Jump: maximum hold duration','Collision avoidance enabled','Collision avoidance: alert distance','Collision avoidance: minimum distance','Jump ceiling','Approach throttle','Always active','Unknown activation-range mode','Alpha-strike probability','Attack throttle','Initial scan trigger: Aim','Initial scan trigger: Hit','Initial scan trigger: Time','Scan/Jump parameter mode'] },
  p9: { title:'Additional Retreat Parameters', description:'Additional retreat behavior; three original meanings have not yet been recovered.', tab:'Retreat', labels:['Unknown','Unknown','Unknown','Retreat on health change enabled','Health-change threshold'] },
  p10:{ title:'Weapon Accuracy', description:'Accuracy settings used while attacking.', tab:'Fire', labels:['Accuracy (0 = most accurate)','Unknown'] },
  p11:{ title:'Fire-Control Multiplier', description:'Multiplier applied to the fire-control or targeting-computer behavior.', tab:'Fire', labels:['FCS / TC multiplier'] },
  p12:{ title:'Vehicle Configuration', description:'Default chassis and mission-start state.', tab:'Vehicle', labels:['Default chassis','Objective-conditional activation','Start shut down'] },
  p13:{ title:'Scan Interruption', description:'Controls whether firing immediately ends an active scan.', tab:'Scan / Jump', labels:['End scan when trigger is pulled'] }
};
export const VLIST = ['Valve state to set','Condition: ammo enabled','Ammo threshold','Condition: fired upon','Condition: health enabled','Health threshold','Condition: heat enabled','Heat threshold','Condition: enters region','Region name','Condition: target LOS enabled','Target-LOS range','Condition: target distance enabled','Target-distance threshold','Condition active','Valve to set','Condition: exits region'];
export const STRATEGIES = ['Direct','Zig Zag','Circle','Jump','Random Move'];
export const CHASSIS = ['notmech','annihilator','avatar','blackhawk','bushwacker','cauldron','champion','daishi','elemental','firefly','madcat','orion','owens','puma','shadowcat','strider','sunder','supernova','thor','vulture'];

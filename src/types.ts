export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };
export interface Vehicle { id: string; name: string; record: Json[]; }
export interface DocumentSnapshot {
  sessionId: number; sourcePath: string; sourceKind: 'json'|'zrd'|'zbd'; archiveEntry?: string;
  vehicles: Vehicle[]; linkedValves: string[]; linkedNets: string[]; warnings: string[];
}
export const GROUPS: Record<string, { title: string; description: string; labels: string[]; tab: string }> = {
  params: { title:'Activation & Engagement', description:'Ranges and timing used when acquiring, approaching, firing at, and leaving a target.', tab:'Activation', labels:['Range to','Enable Initial','Leave Net Range to Target','Return to Net Range from Net','Approach Range to Target','Minimum Range to Target','Maximum Acquisition Range','Fire Time','Cooldown Time'] },
  p2: { title:'Attack Strategy & Retreat Triggers', description:'Movement strategy, strategy-specific distances, and retreat thresholds.', tab:'Attack', labels:['Strategy','Net Distance enabled','Time enabled','Health enabled','Circle Radius / Zig Zag Length','Zig Zag Target Distance','Retreat Time','Retreat Health'] },
  p6: { title:'Timeout Behavior', description:'Controls whether the behavior is disabled permanently or for a limited time.', tab:'Scan / Jump', labels:['Disable','Timeout','For timeout','Forever'] },
  p8: { title:'Combat Behavior', description:'Attack, scan, jump, collision-avoidance, and timing settings.', tab:'Attack', labels:['Min Delay','Max Delay','Min Hold','Max Hold','Collision Avoidance','Alert Distance','Minimum Distance','Jump Ceiling','Attack Approach','Always active','Unknown range-selection value','Alpha Strike Probability','Attack','Initial Aim','Initial Hit','Initial Time','Scan / Jump Mode'] },
  p9: { title:'Additional Retreat Parameters', description:'Retreat-related values; three original meanings have not yet been recovered.', tab:'Retreat', labels:['Unknown','Unknown','Unknown','Health Change enabled','Health Change'] },
  p10:{ title:'Weapon Accuracy', description:'Accuracy settings used while attacking.', tab:'Fire', labels:['Accuracy (0 = most accurate)','Unknown'] },
  p11:{ title:'Fire-Control Multiplier', description:'Multiplier applied to the fire-control or targeting-computer behavior.', tab:'Fire', labels:['FCS / TC multiplier'] },
  p12:{ title:'Vehicle Configuration', description:'Initial chassis and mission-start behavior.', tab:'Vehicle', labels:['Default Chassis','Objective Conditional','Shutdown on Start'] },
  p13:{ title:'Scan Interruption', description:'Controls whether firing immediately ends an active scan.', tab:'Scan / Jump', labels:['Quit scan on trigger pull'] }
};
export const VLIST = ['Close / Open','Ammo enabled','Ammo','Fired on','Health enabled','Health','Heat enabled','Heat','Enters','Region','Target LOS enabled','LOS Range','Target Distance enabled','Target Distance','Active','Valve','Exits'];
export const STRATEGIES = ['Direct','Zig Zag','Circle','Jump','Random Move'];

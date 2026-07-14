export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };
export interface Vehicle { id: string; name: string; record: Json[]; }
export interface DocumentSnapshot {
  sessionId: number; sourcePath: string; sourceKind: 'json'|'zrd'|'zbd'; archiveEntry?: string;
  vehicles: Vehicle[]; linkedValves: string[]; linkedNets: string[]; warnings: string[];
}
export const GROUPS: Record<string, { labels: string[]; tab: string }> = {
  params: { tab:'Activation', labels:['Range to','Enable Initial','Leave Net Range to Target','Return to Net Range from Net','Approach Range to Target','Minimum Range to Target','Maximum Acquisition Range','Fire Time','Cooldown Time'] },
  p2: { tab:'Attack', labels:['Strategy (numeric)','Net Distance enabled','Time enabled','Health enabled','Circle Radius / Zig Zag Length','Zig Zag Target Distance','Retreat Time','Retreat Health'] },
  p6: { tab:'Scan / Jump', labels:['Disable','Timeout','For timeout','Forever'] },
  p8: { tab:'Attack', labels:['Min Delay','Max Delay','Min Hold','Max Hold','Collision Avoidance','Alert Distance','Minimum Distance','Jump Ceiling','Attack Approach','Always active','Range Target Mode','Alpha Strike Probability','Attack','Initial Aim','Initial Hit','Initial Time','Scan / Jump Mode'] },
  p9: { tab:'Retreat', labels:['Unknown p9[0]','Unknown p9[1]','Unknown p9[2]','Health Change enabled','Health Change'] },
  p10:{ tab:'Fire', labels:['Accuracy (0 = most accurate)','Unknown p10[1]'] },
  p11:{ tab:'Fire', labels:['FCS / TC multiplier'] },
  p12:{ tab:'Vehicle', labels:['Default Chassis','Objective Conditional','Shutdown on Start'] },
  p13:{ tab:'Scan / Jump', labels:['Quit scan on trigger pull'] }
};
export const VLIST = ['Close / Open','Ammo enabled','Ammo','Fired on','Health enabled','Health','Heat enabled','Heat','Enters','Region','Target LOS enabled','LOS Range','Target Distance enabled','Target Distance','Active','Valve','Exits'];

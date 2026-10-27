import {describe,expect,it} from 'vitest';
import {badgeWidth,tagBadge} from './network-tags';

describe('network tag badges',()=>{
  it('summarizes the c1m1 net_01 truck records',()=>{
    expect(tagBadge('valve',[1,'truck_go'])).toMatchObject({kind:'condition',text:'⏸ truck_go'});
    expect(tagBadge('valve',[1,'truck_go']).title).toContain('outgoing-link index');
    expect(tagBadge('gate',['ttruck_02'])).toMatchObject({kind:'gate',text:'gate ttruck_02'});
    expect(tagBadge('valve_assign',['destroy_truck_02',1])).toMatchObject({kind:'assign',text:'→ destroy_truck_02 = 1'});
  });
  it('shows explicit targets, disabled conditions, and union members',()=>{
    expect(tagBadge('valve',[0,'alarm',2]).text).toBe('⏸ alarm = 2 (off)');
    expect(tagBadge('valveunion',[[1,'bramp',1],[0,'skip'],[1,'truck_explode']]).text).toBe('⏸ bramp | truck_explode');
    expect(tagBadge('speed',[1,30]).text).toBe('speed');
  });
  it('grows badges for long labels only',()=>{
    expect(badgeWidth('gate')).toBe(54);
    expect(badgeWidth('→ destroy_truck_02 = 1')).toBeGreaterThan(120);
  });
});

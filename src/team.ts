import type { Json } from './types';

export const MAX_TEAM_ID = 31;
export const MAX_CANONICAL_TEAMS = 6;
const UINT32_MAX = 0xffff_ffff;

export interface PackedTeams {
  valid: boolean;
  raw: number | undefined;
  lanes: number[];
  teamIds: number[];
  canonicalRaw: number | undefined;
  canonical: boolean;
  error?: string;
}

export type LegacyRelation = 'Friend' | 'Foe' | 'Other';

export function packTeamIds(teamIds: number[]): number | undefined {
  const ids = [...new Set(teamIds)].sort((a, b) => a - b);
  if (!ids.length || ids.length > MAX_CANONICAL_TEAMS || ids.some(id => !Number.isInteger(id) || id < 0 || id > MAX_TEAM_ID)) return undefined;
  let packed = 0n;
  for (let index = 0; index < ids.length; index++) packed |= BigInt(ids[index]) << BigInt(index * 5);
  return packed <= BigInt(UINT32_MAX) ? Number(packed) : undefined;
}

export function inspectPackedTeams(value: Json | undefined): PackedTeams {
  if (typeof value !== 'number' || !Number.isInteger(value) || value < 0 || value > UINT32_MAX) {
    return { valid: false, raw: undefined, lanes: [], teamIds: [], canonicalRaw: undefined, canonical: false, error: 'Expected an unsigned 32-bit integer.' };
  }
  const lanes: number[] = [];
  let remaining = value;
  do {
    lanes.push(remaining % 32);
    remaining = Math.floor(remaining / 32);
  } while (remaining > 0);
  const teamIds = [...new Set(lanes)].sort((a, b) => a - b);
  const canonicalRaw = packTeamIds(teamIds);
  return { valid: true, raw: value, lanes, teamIds, canonicalRaw, canonical: canonicalRaw === value };
}

export function legacyRelation(teamIds: number[], controlledTeamIds: number[]): LegacyRelation {
  const friendTeams = new Set(controlledTeamIds.filter(id => id !== 0));
  if (teamIds.some(id => friendTeams.has(id))) return 'Friend';
  if (teamIds.some(id => id !== 0 && !friendTeams.has(id))) return 'Foe';
  return 'Other';
}

export function legacyAssignment(relation: LegacyRelation, controlledTeamIds: number[]): number | undefined {
  const friendTeams = new Set(controlledTeamIds.filter(id => id !== 0));
  if (relation === 'Friend') return [...friendTeams].sort((a, b) => a - b)[0];
  if (relation === 'Other') return 0;
  for (let id = 1; id <= MAX_TEAM_ID; id++) if (!friendTeams.has(id)) return id;
  return undefined;
}

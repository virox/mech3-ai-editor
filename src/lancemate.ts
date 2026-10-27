import type { Json } from './types';

export type LancemateState = 'auto' | 'on' | 'off' | 'unknown';

export interface LancemateValue {
  state: LancemateState;
  present: boolean;
  stored: string;
}

export function inspectLancemate(value: Json | undefined): LancemateValue {
  if (value === undefined) return { state: 'auto', present: false, stored: 'Tag absent (Auto default)' };
  if (!Array.isArray(value) || value.length === 0) return { state: 'unknown', present: true, stored: JSON.stringify(value) };

  const raw = value[0];
  if (raw === 0) return { state: 'off', present: true, stored: '[0]' };
  if (raw === 1) return { state: 'on', present: true, stored: '[1]' };
  if (raw === 2) return { state: 'auto', present: true, stored: '[2] (explicit Auto)' };
  if (typeof raw === 'string') {
    const normalized = raw.trim().toLowerCase();
    if (normalized === 'on' || normalized === 'true') return { state: 'on', present: true, stored: JSON.stringify(value) };
    if (normalized === 'off' || normalized === 'false' || normalized === 'none' || normalized === 'default') return { state: 'off', present: true, stored: JSON.stringify(value) };
  }
  return { state: 'unknown', present: true, stored: JSON.stringify(value) };
}

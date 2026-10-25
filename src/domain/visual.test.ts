import { describe, expect, it } from 'vitest';
import { palette } from './visual';
import type { BrandKit } from './types';
const brand: BrandKit = { id: 'b', name: 'B', sector: '', description: '', colors: [{ name: '', hex: '#eeeeee', role: '' }, { name: '', hex: '#111111', role: '' }, { name: '', hex: '#cc5533', role: '' }], displayFont: 'Arial', bodyFont: 'Arial', tone: [], dos: [], donts: [], createdAt: '' };
describe('palette', () => { it('maps a dark version to contrast-safe brand colours', () => expect(palette(brand, 'dark')).toMatchObject({ bg: '#111111', fg: '#eeeeee', shape: '#cc5533' })); });

export type Format = 'post' | 'story' | 'square' | 'landscape';
export type ProviderKind = 'codex_cli' | 'claude_cli' | 'open_ai_compatible' | 'anthropic';
export interface BrandColor { name: string; hex: string; role: string }
export interface BrandKit { id: string; name: string; sector: string; description: string; colors: BrandColor[]; displayFont: string; bodyFont: string; tone: string[]; dos: string[]; donts: string[]; createdAt: string }
export interface VisualSpec { headline: string; imageData: string; kicker: string; subline: string; details: string[]; cta: string; footer: string; composition: 'editorial' | 'event' | 'split' | 'typographic'; referenceNote: string; scheme: 'light' | 'dark' | 'accent' | 'secondary'; alignment: 'left' | 'center'; titleSize: number; shape: string }
export interface VisualVersion { id: string; projectId: string; sequence: number; label: string; prompt: string; spec: VisualSpec; createdAt: string }
export interface Attachment { name: string; mimeType: string; dataBase64: string; previewUrl?: string }
export interface Project { id: string; title: string; brandKitId: string; format: Format; createdAt: string; updatedAt: string; versions: VisualVersion[] }
export interface ProjectSummary { id: string; title: string; brandKitId: string; format: Format; createdAt: string; updatedAt: string; latestVersion?: VisualVersion }
export interface Provider { id: string; kind: ProviderKind; name: string; model: string; endpoint?: string; enabled: boolean; hasSecret: boolean; createdAt: string }
export interface ProviderInput { id?: string; kind: ProviderKind; name: string; model: string; endpoint?: string; apiKey?: string; enabled: boolean }
export interface Bootstrap { projects: ProjectSummary[]; brandKits: BrandKit[]; providers: Provider[] }

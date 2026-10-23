export type ViewType = 
  | 'patch'
  | 'cues'
  | 'groups'
  | 'effects'
  | 'presets'
  | 'output-monitor'
  | 'executors'
  | 'command';

export interface TabConfig {
  id: string;
  type: ViewType;
  label: string;
  shortLabel: string;
  priority: number; // 1 = highest, 3 = lowest
  pinned?: boolean;
}

export interface PaneConfig {
  id: string;
  tabs: TabConfig[];
  activeTabId: string;
}

export interface SplitConfig {
  id: string;
  direction: 'horizontal' | 'vertical';
  sizes: number[]; // Percentages
  children: (PaneConfig | SplitConfig)[];
}

export type LayoutConfig = PaneConfig | SplitConfig | null;

export function isPaneConfig(layout: any): layout is PaneConfig {
  return layout && 'tabs' in layout && 'activeTabId' in layout;
}

export function isSplitConfig(layout: any): layout is SplitConfig {
  return layout && 'direction' in layout && 'children' in layout;
}
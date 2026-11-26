export type SupportedLocale =
  | 'zh-CN'
  | 'zh-TW'
  | 'en-US'
  | 'ja-JP'
  | 'ko-KR'
  | 'fr-FR'
  | 'de-DE'
  | 'es-ES'
  | 'ru-RU'
  | 'pt-BR';

export type ModifierKey = 'Command' | 'Control' | 'Alt';

export type BuiltinAction = 'copy_path' | 'open_in_terminal';

export type TerminalAdapter =
  | 'Terminal'
  | 'iTerm2'
  | 'Warp'
  | 'WindowsTerminal'
  | 'PowerShell'
  | 'GnomeTerminal'
  | 'Alacritty';

export type HostPlatform = 'macOS' | 'Windows' | 'Linux';

export interface AppSlotItem {
  id: string;
  type: 'app';
  name: string;
  execPath: string;
  iconCache: string;
  presetKey?: string;
  customIconDataUrl?: string;
}

export interface BuiltinSlotItem {
  id: string;
  type: 'builtin';
  action: BuiltinAction;
}

export type ActiveSlot = AppSlotItem | BuiltinSlotItem;

export interface LibraryApp {
  id: string;
  name: string;
  execPath: string;
  iconCache: string;
  presetKey?: string;
  customIconDataUrl?: string;
  category?: 'ide' | 'editor' | 'system' | 'custom';
}

export interface KeyRingConfig {
  language: SupportedLocale;
  theme?: 'light' | 'dark';
  trigger: {
    modifier: ModifierKey;
    intervalMs: number;
  };
  ui: {
    ringDiameter: number;
    innerHoleDiameter: number;
    iconSize: number;
  };
  autostart: boolean;
  terminalApp: TerminalAdapter;
  activeSlots: ActiveSlot[];
  libraryApps: LibraryApp[];
}

export interface ContextEntry {
  id: string;
  name: string;
  path: string;
  kind: 'folder' | 'file';
  size?: string;
  modified?: string;
}

export interface ContextState {
  currentDirectory: string;
  selectedPaths: string[];
  selectedEntries: ContextEntry[];
  sourceApp: string;
}

export interface ExecutionRecord {
  id: string;
  timestamp: string;
  slotId: string;
  label: string;
  actionType: 'copy_path' | 'open_in_terminal' | 'launch_app';
  commandPreview: string;
  targetPaths: string[];
  workingDirectory: string;
}

export type StudioPage = 'composer' | 'library' | 'preferences' | 'sandbox' | 'source';

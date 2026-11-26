import { ContextEntry, KeyRingConfig, LibraryApp, TerminalAdapter } from '../types/keyring';

export const DEFAULT_CONFIG: KeyRingConfig = {
  language: 'zh-CN',
  theme: 'light',
  trigger: {
    modifier: 'Command',
    intervalMs: 300,
  },
  ui: {
    ringDiameter: 260,
    innerHoleDiameter: 84,
    iconSize: 36,
  },
  autostart: true,
  terminalApp: 'Terminal',
  activeSlots: [
    {
      id: 'app-cursor',
      type: 'app',
      name: 'Cursor',
      execPath: '/Applications/Cursor.app',
      iconCache: 'icons/cursor.png',
      presetKey: 'cursor',
    },
    {
      id: 'app-vscode',
      type: 'app',
      name: 'Visual Studio Code',
      execPath: '/Applications/Visual Studio Code.app',
      iconCache: 'icons/vscode.png',
      presetKey: 'vscode',
    },
    {
      id: 'builtin-terminal',
      type: 'builtin',
      action: 'open_in_terminal',
    },
    {
      id: 'builtin-copy',
      type: 'builtin',
      action: 'copy_path',
    },
  ],
  libraryApps: [
    {
      id: 'app-zed',
      name: 'Zed',
      execPath: '/Applications/Zed.app',
      iconCache: 'icons/zed.png',
      presetKey: 'zed',
      category: 'editor',
    },
    {
      id: 'app-webstorm',
      name: 'WebStorm',
      execPath: '/Applications/WebStorm.app',
      iconCache: 'icons/webstorm.png',
      presetKey: 'webstorm',
      category: 'ide',
    },
  ],
};

export const COMMON_IDE_PRESETS: LibraryApp[] = [
  {
    id: 'app-cursor',
    name: 'Cursor',
    execPath: '/Applications/Cursor.app',
    iconCache: 'icons/cursor.png',
    presetKey: 'cursor',
    category: 'ide',
  },
  {
    id: 'app-vscode',
    name: 'Visual Studio Code',
    execPath: '/Applications/Visual Studio Code.app',
    iconCache: 'icons/vscode.png',
    presetKey: 'vscode',
    category: 'ide',
  },
  {
    id: 'app-intellij',
    name: 'IntelliJ IDEA',
    execPath: '/Applications/IntelliJ IDEA.app',
    iconCache: 'icons/intellij.png',
    presetKey: 'intellij',
    category: 'ide',
  },
  {
    id: 'app-webstorm',
    name: 'WebStorm',
    execPath: '/Applications/WebStorm.app',
    iconCache: 'icons/webstorm.png',
    presetKey: 'webstorm',
    category: 'ide',
  },
  {
    id: 'app-pycharm',
    name: 'PyCharm',
    execPath: '/Applications/PyCharm.app',
    iconCache: 'icons/pycharm.png',
    presetKey: 'pycharm',
    category: 'ide',
  },
  {
    id: 'app-sublime',
    name: 'Sublime Text',
    execPath: '/Applications/Sublime Text.app',
    iconCache: 'icons/sublime.png',
    presetKey: 'sublime',
    category: 'editor',
  },
  {
    id: 'app-zed',
    name: 'Zed',
    execPath: '/Applications/Zed.app',
    iconCache: 'icons/zed.png',
    presetKey: 'zed',
    category: 'editor',
  },
  {
    id: 'app-windsurf',
    name: 'Windsurf',
    execPath: '/Applications/Windsurf.app',
    iconCache: 'icons/windsurf.png',
    presetKey: 'windsurf',
    category: 'ide',
  },
  {
    id: 'app-xcode',
    name: 'Xcode',
    execPath: '/Applications/Xcode.app',
    iconCache: 'icons/xcode.png',
    presetKey: 'xcode',
    category: 'ide',
  },
  {
    id: 'app-obsidian',
    name: 'Obsidian',
    execPath: '/Applications/Obsidian.app',
    iconCache: 'icons/obsidian.png',
    presetKey: 'obsidian',
    category: 'editor',
  },
];

export const TERMINAL_ADAPTER_OPTIONS: {
  id: TerminalAdapter;
  label: string;
  platform: string;
  commandTemplate: string;
}[] = [
  { id: 'Terminal', label: 'Terminal.app', platform: 'macOS', commandTemplate: 'open -a Terminal "{cwd}"' },
  { id: 'iTerm2', label: 'iTerm2', platform: 'macOS', commandTemplate: 'open -a iTerm "{cwd}"' },
  { id: 'Warp', label: 'Warp', platform: 'macOS', commandTemplate: 'open -a Warp "{cwd}"' },
  { id: 'WindowsTerminal', label: 'Windows Terminal (wt.exe)', platform: 'Windows', commandTemplate: 'wt.exe -d "{cwd}"' },
  { id: 'PowerShell', label: 'PowerShell', platform: 'Windows', commandTemplate: 'powershell.exe -NoExit -Command "Set-Location \'{cwd}\'"' },
  { id: 'GnomeTerminal', label: 'GNOME Terminal', platform: 'Linux', commandTemplate: 'gnome-terminal --working-directory="{cwd}"' },
  { id: 'Alacritty', label: 'Alacritty', platform: 'Linux / macOS', commandTemplate: 'alacritty --working-directory "{cwd}"' },
];

export const SANDBOX_ROOT_DIR = '/Users/alex/Projects/keyring-app';

export const SANDBOX_ENTRIES: ContextEntry[] = [
  {
    id: 'e-1',
    name: 'src-tauri',
    path: '/Users/alex/Projects/keyring-app/src-tauri',
    kind: 'folder',
    size: '8 items',
    modified: 'Today 21:40',
  },
  {
    id: 'e-2',
    name: 'src',
    path: '/Users/alex/Projects/keyring-app/src',
    kind: 'folder',
    size: '14 items',
    modified: 'Today 21:55',
  },
  {
    id: 'e-3',
    name: 'package.json',
    path: '/Users/alex/Projects/keyring-app/package.json',
    kind: 'file',
    size: '1.4 KB',
    modified: 'Today 18:12',
  },
  {
    id: 'e-4',
    name: 'Cargo.toml',
    path: '/Users/alex/Projects/keyring-app/src-tauri/Cargo.toml',
    kind: 'file',
    size: '920 B',
    modified: 'Yesterday',
  },
  {
    id: 'e-5',
    name: 'hotkey.rs',
    path: '/Users/alex/Projects/keyring-app/src-tauri/src/hotkey.rs',
    kind: 'file',
    size: '4.8 KB',
    modified: 'Today 20:05',
  },
  {
    id: 'e-6',
    name: 'README.md',
    path: '/Users/alex/Projects/keyring-app/README.md',
    kind: 'file',
    size: '6.2 KB',
    modified: 'Nov 22',
  },
];

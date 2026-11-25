import React, { useState } from 'react';
import { Check, Copy } from 'lucide-react';
import { TranslationDict } from '../i18n/translations';

interface TauriSourcePageProps {
  t: TranslationDict;
}

interface SourceFileEntry {
  path: string;
  description: string;
  content: string;
}

const SOURCE_FILES: SourceFileEntry[] = [
  {
    path: 'src-tauri/tauri.conf.json',
    description: 'PieWindow (transparent, alwaysOnTop, decorations: false) & ConfigWindow (titleBarStyle: Overlay)',
    content: `{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "KeyRing",
  "version": "0.1.0",
  "identifier": "app.keyring.launcher",
  "app": {
    "macOSPrivateApi": true,
    "windows": [
      {
        "label": "pie",
        "url": "/?window=pie",
        "width": 340,
        "height": 340,
        "decorations": false,
        "transparent": true,
        "alwaysOnTop": true,
        "skipTaskbar": true,
        "visible": false
      },
      {
        "label": "config",
        "url": "/?window=config",
        "width": 1060,
        "height": 720,
        "decorations": true,
        "titleBarStyle": "Overlay",
        "hiddenTitle": true,
        "visible": false
      }
    ]
  }
}`,
  },
  {
    path: 'src-tauri/src/hotkey.rs',
    description: 'rdev background thread with Press → Release anti-mistouch state machine & macOS AXIsProcessTrusted check',
    content: `// Generated at /src-tauri/src/hotkey.rs
// Tracks complete Press -> Release cycles for Command / Control / Alt.
// Any other key press or mouse button click resets state immediately.
// When 2 clean Releases occur within interval_ms (default 300ms),
// triggers on_trigger(pointer_x, pointer_y).`,
  },
  {
    path: 'src-tauri/src/context.rs',
    description: 'macOS osascript Finder selection, Windows IShellWindows COM, Linux D-Bus FileManager1',
    content: `// Generated at /src-tauri/src/context.rs
// Zero-clipboard-pollution active selection capture:
// - macOS: osascript reading Finder selection POSIX paths or target of front window
// - Windows: IShellWindows COM inspecting FolderItem.Path of foreground HWND
// - Linux: D-Bus org.freedesktop.FileManager1 / X11 PRIMARY URI decoding`,
  },
  {
    path: 'src-tauri/src/icon.rs',
    description: 'Native icon extraction (macOS Info.plist + icns -> 128x128 PNG, Windows SHGetFileInfoW, Linux .desktop)',
    content: `// Generated at /src-tauri/src/icon.rs
// Extracts 128x128 RGBA PNG icons cached locally as Base64 data URLs.`,
  },
  {
    path: 'src-tauri/src/executor.rs',
    description: 'copy_path (clipboard plugin), open_in_terminal (folder or parent dir), and app launch',
    content: `// Generated at /src-tauri/src/executor.rs
// - copy_path: writes newline-joined paths via tauri-plugin-clipboard-manager
// - open_in_terminal: launches Terminal / iTerm2 / Warp / wt.exe / gnome-terminal
// - external app: passes effective context paths as launch arguments`,
  },
  {
    path: 'src-tauri/src/tray.rs',
    description: 'Dynamic System Tray builder localized across all 10 supported locales',
    content: `// Generated at /src-tauri/src/tray.rs
// Rebuilds tray menu immediately when config.language changes.`,
  },
];

export const TauriSourcePage: React.FC<TauriSourcePageProps> = ({ t }) => {
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [copied, setCopied] = useState(false);

  const activeFile = SOURCE_FILES[selectedIndex];

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(activeFile.content);
      setCopied(true);
      setTimeout(() => setCopied(false), 1600);
    } catch {
      // ignore
    }
  };

  return (
    <div className="max-w-4xl mx-auto px-6 md:px-14 py-10 md:py-12">
      <header className="pb-8 border-b border-neutral-200/80 dark:border-neutral-800">
        <h1 className="text-[28px] font-semibold tracking-tight text-neutral-900 dark:text-neutral-100 leading-tight">
          {t.source.title}
        </h1>
        <p className="mt-2 text-[14px] text-neutral-500 dark:text-neutral-400 leading-relaxed max-w-2xl">
          {t.source.subtitle}
        </p>
      </header>

      <div className="mt-8 grid grid-cols-1 md:grid-cols-12 gap-6 items-start">
        <div className="md:col-span-4 space-y-1">
          {SOURCE_FILES.map((file, idx) => {
            const active = idx === selectedIndex;
            return (
              <button
                key={file.path}
                onClick={() => setSelectedIndex(idx)}
                className={`w-full text-left px-3 py-2 rounded-lg text-[12.5px] font-mono transition-colors truncate ${
                  active
                    ? 'bg-[#EAEAE6] dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 font-medium'
                    : 'text-neutral-600 dark:text-neutral-400 hover:bg-[#F7F7F5] dark:hover:bg-neutral-800/60'
                }`}
              >
                {file.path}
              </button>
            );
          })}
        </div>

        <div className="md:col-span-8 rounded-xl border border-neutral-200/80 dark:border-neutral-800 bg-[#FAFAF9] dark:bg-[#171717] overflow-hidden">
          <div className="px-4 py-3 border-b border-neutral-200/80 dark:border-neutral-800 flex items-center justify-between gap-3">
            <div className="min-w-0">
              <div className="text-[13px] font-mono font-medium text-neutral-900 dark:text-neutral-100 truncate">
                {activeFile.path}
              </div>
              <div className="text-[12px] text-neutral-500 dark:text-neutral-400 truncate mt-0.5">
                {activeFile.description}
              </div>
            </div>
            <button
              onClick={handleCopy}
              className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[12px] border border-neutral-200 dark:border-neutral-700 bg-white dark:bg-neutral-800 text-neutral-700 dark:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-700 shrink-0"
            >
              {copied ? <Check className="w-3.5 h-3.5 text-emerald-600" /> : <Copy className="w-3.5 h-3.5" />}
              <span>{copied ? t.source.copied : t.source.copyFile}</span>
            </button>
          </div>
          <pre className="p-4 text-[12px] font-mono text-neutral-700 dark:text-neutral-300 overflow-x-auto leading-relaxed">
            {activeFile.content}
          </pre>
        </div>
      </div>
    </div>
  );
};

import { useCallback, useEffect, useState } from 'react';
import { Check } from 'lucide-react';
import {
  ActiveSlot,
  BuiltinAction,
  ContextEntry,
  ContextState,
  ExecutionRecord,
  KeyRingConfig,
  LibraryApp,
  StudioPage,
  TerminalAdapter,
} from './types/keyring';
import { TRANSLATIONS } from './i18n/translations';
import {
  DEFAULT_CONFIG,
  SANDBOX_ENTRIES,
  SANDBOX_ROOT_DIR,
  TERMINAL_ADAPTER_OPTIONS,
} from './data/presets';
import { useDoubleTapTrigger } from './hooks/useDoubleTapTrigger';
import { Sidebar } from './components/Sidebar';
import { RingComposerPage } from './components/RingComposerPage';
import { LibraryPage } from './components/LibraryPage';
import { PreferencesPage } from './components/PreferencesPage';
import { SandboxPage } from './components/SandboxPage';
import { TauriSourcePage } from './components/TauriSourcePage';
import { PieMenuOverlay } from './components/PieMenuOverlay';
import { getSlotDisplayName } from './components/PieSvgRing';
import {
  executeNativeSlot,
  getWindowMode,
  hidePieWindow,
  persistNativeConfig,
} from './tauriBridge';

const CONFIG_STORAGE_KEY = 'keyring_config_v1';
const THEME_STORAGE_KEY = 'keyring_theme_v1';

function loadInitialConfig(): KeyRingConfig {
  try {
    const raw = localStorage.getItem(CONFIG_STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw) as Partial<KeyRingConfig>;
      if (parsed && parsed.trigger && parsed.ui && Array.isArray(parsed.activeSlots)) {
        return {
          ...DEFAULT_CONFIG,
          ...parsed,
          trigger: { ...DEFAULT_CONFIG.trigger, ...parsed.trigger },
          ui: { ...DEFAULT_CONFIG.ui, ...parsed.ui },
          libraryApps: Array.isArray(parsed.libraryApps)
            ? parsed.libraryApps
            : DEFAULT_CONFIG.libraryApps,
        };
      }
    }
  } catch {
    // fallback to defaults
  }
  return DEFAULT_CONFIG;
}

export default function App() {
  const windowMode = getWindowMode();
  const [config, setConfig] = useState<KeyRingConfig>(loadInitialConfig);
  const [activePage, setActivePage] = useState<StudioPage>('composer');
  const [darkMode, setDarkMode] = useState<boolean>(() => {
    try {
      return localStorage.getItem(THEME_STORAGE_KEY) === 'dark';
    } catch {
      return false;
    }
  });
  const [mobileSidebarOpen, setMobileSidebarOpen] = useState(false);
  const [listenerPaused, setListenerPaused] = useState(false);

  // Floating Pie Menu State
  const [pieVisible, setPieVisible] = useState(() => windowMode === 'pie');
  const [pieCoords, setPieCoords] = useState<{ x: number; y: number }>(() => {
    if (windowMode === 'pie' && typeof window !== 'undefined') {
      return { x: Math.round(window.innerWidth / 2), y: Math.round(window.innerHeight / 2) };
    }
    return { x: 600, y: 360 };
  });

  // Context-Aware Sandbox State (default first folder selected for immediate testing)
  const [selectedEntries, setSelectedEntries] = useState<ContextEntry[]>([SANDBOX_ENTRIES[0]]);
  const [executionLogs, setExecutionLogs] = useState<ExecutionRecord[]>([]);
  const [activeTerminalSession, setActiveTerminalSession] = useState<{
    cwd: string;
    command: string;
  } | null>(null);

  // Toast feedback
  const [toast, setToast] = useState<{ title: string; detail: string } | null>(null);

  const t = TRANSLATIONS[config.language] || TRANSLATIONS['zh-CN'];

  useEffect(() => {
    try {
      localStorage.setItem(CONFIG_STORAGE_KEY, JSON.stringify(config));
    } catch {
      // ignore quota errors
    }
    void persistNativeConfig(config);
  }, [config]);

  useEffect(() => {
    document.documentElement.classList.toggle('dark', darkMode);
    try {
      localStorage.setItem(THEME_STORAGE_KEY, darkMode ? 'dark' : 'light');
    } catch {
      // ignore
    }
  }, [darkMode]);

  useEffect(() => {
    if (windowMode !== 'pie') {
      return;
    }
    const previousHtml = document.documentElement.style.background;
    const previousBody = document.body.style.background;
    document.documentElement.style.background = 'transparent';
    document.body.style.background = 'transparent';
    document.body.classList.remove('bg-white');
    return () => {
      document.documentElement.style.background = previousHtml;
      document.body.style.background = previousBody;
    };
  }, [windowMode]);

  const showToast = useCallback((title: string, detail: string) => {
    setToast({ title, detail });
    window.setTimeout(() => {
      setToast((cur) => (cur?.title === title && cur?.detail === detail ? null : cur));
    }, 2800);
  }, []);

  const summonPieAt = useCallback((coords: { x: number; y: number }) => {
    setPieCoords(coords);
    setPieVisible(true);
  }, []);

  const { phase, lastEventNote, lastDeltaMs, getPointerCoords } = useDoubleTapTrigger({
    modifier: config.trigger.modifier,
    intervalMs: config.trigger.intervalMs,
    enabled: windowMode !== 'pie' && !listenerPaused && !pieVisible,
    onTrigger: summonPieAt,
  });

  const contextState: ContextState = {
    currentDirectory: SANDBOX_ROOT_DIR,
    selectedPaths: selectedEntries.map((e) => e.path),
    selectedEntries,
    sourceApp: 'Finder',
  };

  const resolveWorkingDirectory = useCallback((): string => {
    if (selectedEntries.length > 0) {
      const first = selectedEntries[0];
      if (first.kind === 'folder') {
        return first.path;
      }
      const lastSlash = first.path.lastIndexOf('/');
      return lastSlash > 0 ? first.path.slice(0, lastSlash) : SANDBOX_ROOT_DIR;
    }
    return SANDBOX_ROOT_DIR;
  }, [selectedEntries]);

  const closePieMenu = useCallback(() => {
    setPieVisible(false);
    if (windowMode === 'pie') {
      void hidePieWindow();
    }
  }, [windowMode]);

  const handleExecuteSlot = useCallback(
    async (slot: ActiveSlot) => {
      if (windowMode === 'pie') {
        const handled = await executeNativeSlot(slot.id);
        await hidePieWindow();
        setPieVisible(false);
        if (handled) {
          return;
        }
      }
      setPieVisible(false);
      const effectivePaths =
        contextState.selectedPaths.length > 0
          ? contextState.selectedPaths
          : [contextState.currentDirectory];
      const nowStr = new Date().toLocaleTimeString([], {
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
      });
      const label = getSlotDisplayName(slot, t);

      if (slot.type === 'builtin' && slot.action === 'copy_path') {
        const joined = effectivePaths.join('\n');
        try {
          await navigator.clipboard.writeText(joined);
        } catch {
          // ignore if clipboard permission is restricted
        }
        showToast(t.pie.copiedToast, joined);
        setExecutionLogs((prev) => [
          {
            id: `log-${Date.now()}`,
            timestamp: nowStr,
            slotId: slot.id,
            label,
            actionType: 'copy_path',
            commandPreview: `clipboard.write_text("${joined.replace(/\n/g, '\\n')}")`,
            targetPaths: effectivePaths,
            workingDirectory: resolveWorkingDirectory(),
          },
          ...prev.slice(0, 14),
        ]);
        return;
      }

      if (slot.type === 'builtin' && slot.action === 'open_in_terminal') {
        const cwd = resolveWorkingDirectory();
        const adapter =
          TERMINAL_ADAPTER_OPTIONS.find((opt) => opt.id === config.terminalApp) ||
          TERMINAL_ADAPTER_OPTIONS[0];
        const cmd = adapter.commandTemplate.replace('{cwd}', cwd);
        setActiveTerminalSession({ cwd, command: cmd });
        showToast(t.pie.terminalLaunchedToast, `${adapter.label} · ${cwd}`);
        setExecutionLogs((prev) => [
          {
            id: `log-${Date.now()}`,
            timestamp: nowStr,
            slotId: slot.id,
            label,
            actionType: 'open_in_terminal',
            commandPreview: cmd,
            targetPaths: effectivePaths,
            workingDirectory: cwd,
          },
          ...prev.slice(0, 14),
        ]);
        return;
      }

      if (slot.type === 'app') {
        const args = effectivePaths.map((p) => `"${p}"`).join(' ');
        const cmd = `open -a "${slot.execPath}" ${args}`;
        showToast(t.pie.appLaunchedToast, `${slot.name} · ${effectivePaths[0]}`);
        setExecutionLogs((prev) => [
          {
            id: `log-${Date.now()}`,
            timestamp: nowStr,
            slotId: slot.id,
            label,
            actionType: 'launch_app',
            commandPreview: cmd,
            targetPaths: effectivePaths,
            workingDirectory: resolveWorkingDirectory(),
          },
          ...prev.slice(0, 14),
        ]);
      }
    },
    [
      config.terminalApp,
      contextState.currentDirectory,
      contextState.selectedPaths,
      resolveWorkingDirectory,
      showToast,
      t,
      windowMode,
    ]
  );

  // Ring & Library Mutations
  const handleReorderSlots = (nextSlots: ActiveSlot[]) => {
    setConfig((prev) => ({ ...prev, activeSlots: nextSlots }));
  };

  const handleRemoveSlot = (slotId: string) => {
    setConfig((prev) => {
      const target = prev.activeSlots.find((s) => s.id === slotId);
      if (!target) return prev;
      const nextActive = prev.activeSlots.filter((s) => s.id !== slotId);
      let nextLibrary = prev.libraryApps;

      if (target.type === 'app') {
        const alreadyInLib = prev.libraryApps.some(
          (a) => a.id === target.id || a.execPath === target.execPath
        );
        if (!alreadyInLib) {
          nextLibrary = [
            {
              id: target.id,
              name: target.name,
              execPath: target.execPath,
              iconCache: target.iconCache,
              presetKey: target.presetKey,
              customIconDataUrl: target.customIconDataUrl,
            },
            ...prev.libraryApps,
          ];
        }
      }

      return {
        ...prev,
        activeSlots: nextActive,
        libraryApps: nextLibrary,
      };
    });
  };

  const handleAddBuiltinToRing = (action: BuiltinAction) => {
    setConfig((prev) => {
      if (prev.activeSlots.length >= 8) return prev;
      if (prev.activeSlots.some((s) => s.type === 'builtin' && s.action === action)) {
        return prev;
      }
      const newSlot: ActiveSlot = {
        id: action === 'copy_path' ? 'builtin-copy' : 'builtin-terminal',
        type: 'builtin',
        action,
      };
      return {
        ...prev,
        activeSlots: [...prev.activeSlots, newSlot],
      };
    });
  };

  const handleAddAppToRing = (app: LibraryApp) => {
    setConfig((prev) => {
      if (prev.activeSlots.length >= 8) return prev;
      if (
        prev.activeSlots.some(
          (s) => s.type === 'app' && (s.id === app.id || s.execPath === app.execPath)
        )
      ) {
        return prev;
      }
      const newSlot: ActiveSlot = {
        id: app.id,
        type: 'app',
        name: app.name,
        execPath: app.execPath,
        iconCache: app.iconCache,
        presetKey: app.presetKey,
        customIconDataUrl: app.customIconDataUrl,
      };
      return {
        ...prev,
        activeSlots: [...prev.activeSlots, newSlot],
        libraryApps: prev.libraryApps.filter(
          (a) => a.id !== app.id && a.execPath !== app.execPath
        ),
      };
    });
  };

  const handleAddAppToLibrary = (app: LibraryApp) => {
    setConfig((prev) => {
      if (prev.activeSlots.length < 8) {
        return {
          ...prev,
          activeSlots: [
            ...prev.activeSlots,
            {
              id: app.id,
              type: 'app',
              name: app.name,
              execPath: app.execPath,
              iconCache: app.iconCache,
              presetKey: app.presetKey,
              customIconDataUrl: app.customIconDataUrl,
            },
          ],
        };
      }
      return {
        ...prev,
        libraryApps: [app, ...prev.libraryApps],
      };
    });
  };

  const handleDeleteLibraryApp = (appId: string) => {
    setConfig((prev) => ({
      ...prev,
      libraryApps: prev.libraryApps.filter((a) => a.id !== appId),
    }));
  };

  const handleImportMissingIdes = (apps: LibraryApp[]) => {
    setConfig((prev) => {
      const existingIds = new Set([
        ...prev.activeSlots.map((s) => s.id),
        ...prev.libraryApps.map((a) => a.id),
      ]);
      const toAdd = apps.filter((a) => !existingIds.has(a.id));
      return {
        ...prev,
        libraryApps: [...prev.libraryApps, ...toAdd],
      };
    });
  };

  const handleToggleSandboxEntry = (entry: ContextEntry, multiSelect: boolean) => {
    setSelectedEntries((prev) => {
      const exists = prev.some((e) => e.id === entry.id);
      if (multiSelect) {
        return exists ? prev.filter((e) => e.id !== entry.id) : [...prev, entry];
      }
      return exists && prev.length === 1 ? [] : [entry];
    });
  };

  const triggerCenterPopup = () => {
    const cx = typeof window !== 'undefined' ? Math.round(window.innerWidth / 2) : 640;
    const cy = typeof window !== 'undefined' ? Math.round(window.innerHeight / 2) : 400;
    summonPieAt({ x: cx, y: cy });
  };

  const triggerAtCursorOrCenter = () => {
    const ptr = getPointerCoords();
    if (ptr.x > 0 && ptr.y > 0) {
      summonPieAt(ptr);
    } else {
      triggerCenterPopup();
    }
  };

  if (windowMode === 'pie') {
    return (
      <div className="w-screen h-screen bg-transparent overflow-hidden">
        <PieMenuOverlay
          visible={pieVisible}
          coords={pieCoords}
          slots={config.activeSlots}
          ringDiameter={config.ui.ringDiameter}
          innerHoleDiameter={config.ui.innerHoleDiameter}
          iconSize={config.ui.iconSize}
          t={t}
          darkSurface={darkMode}
          onExecute={handleExecuteSlot}
          onClose={closePieMenu}
        />
      </div>
    );
  }

  return (
    <div className="min-h-screen w-full flex bg-white dark:bg-[#1F1F1F] text-[#1F1F1F] dark:text-[#EDEDEC]">
      {/* Frameless Notion-style Sidebar (No OS Top Bar) */}
      <Sidebar
        activePage={activePage}
        onNavigate={setActivePage}
        t={t}
        activeSlotCount={config.activeSlots.length}
        modifier={config.trigger.modifier}
        intervalMs={config.trigger.intervalMs}
        autostart={config.autostart}
        onToggleAutostart={() =>
          setConfig((prev) => ({ ...prev, autostart: !prev.autostart }))
        }
        listenerPaused={listenerPaused}
        onToggleListenerPaused={() => setListenerPaused((p) => !p)}
        darkMode={darkMode}
        onToggleDarkMode={() => setDarkMode((d) => !d)}
        onTestPopup={triggerAtCursorOrCenter}
        mobileOpen={mobileSidebarOpen}
        onSetMobileOpen={setMobileSidebarOpen}
      />

      {/* Main Notion Document Canvas */}
      <main className="flex-1 min-w-0 overflow-y-auto">
        {activePage === 'composer' && (
          <RingComposerPage
            config={config}
            t={t}
            darkMode={darkMode}
            onReorderSlots={handleReorderSlots}
            onRemoveSlot={handleRemoveSlot}
            onExecuteSlot={handleExecuteSlot}
            onPopoutTest={triggerCenterPopup}
            onNavigateLibrary={() => setActivePage('library')}
          />
        )}

        {activePage === 'library' && (
          <LibraryPage
            config={config}
            t={t}
            onAddBuiltinToRing={handleAddBuiltinToRing}
            onRemoveSlot={handleRemoveSlot}
            onAddAppToRing={handleAddAppToRing}
            onAddAppToLibrary={handleAddAppToLibrary}
            onDeleteLibraryApp={handleDeleteLibraryApp}
            onChangeTerminalApp={(adapter: TerminalAdapter) =>
              setConfig((prev) => ({ ...prev, terminalApp: adapter }))
            }
            onImportMissingIdes={handleImportMissingIdes}
          />
        )}

        {activePage === 'preferences' && (
          <PreferencesPage
            config={config}
            t={t}
            triggerPhase={phase}
            lastEventNote={lastEventNote}
            lastDeltaMs={lastDeltaMs}
            onUpdateConfig={setConfig}
            onResetDefaults={() => setConfig(DEFAULT_CONFIG)}
            onImportConfig={(imported) => setConfig(imported)}
          />
        )}

        {activePage === 'sandbox' && (
          <SandboxPage
            config={config}
            t={t}
            contextState={contextState}
            triggerPhase={phase}
            lastEventNote={lastEventNote}
            onChangeModifier={(modifier) =>
              setConfig((prev) => ({
                ...prev,
                trigger: { ...prev.trigger, modifier },
              }))
            }
            onToggleEntry={handleToggleSandboxEntry}
            onClearSelection={() => setSelectedEntries([])}
            onTriggerAtCenter={triggerCenterPopup}
            executionLogs={executionLogs}
            activeTerminalSession={activeTerminalSession}
            onCloseTerminal={() => setActiveTerminalSession(null)}
          />
        )}

        {activePage === 'source' && <TauriSourcePage t={t} />}
      </main>

      {/* Floating Radial Pie Menu Overlay */}
      <PieMenuOverlay
        visible={pieVisible}
        coords={pieCoords}
        slots={config.activeSlots}
        ringDiameter={config.ui.ringDiameter}
        innerHoleDiameter={config.ui.innerHoleDiameter}
        iconSize={config.ui.iconSize}
        t={t}
        darkSurface={darkMode}
        onExecute={handleExecuteSlot}
        onClose={closePieMenu}
      />

      {/* Lightweight Execution Notification Toast */}
      {toast && (
        <div className="fixed bottom-5 right-5 z-50 max-w-sm rounded-xl border border-neutral-200 dark:border-neutral-700 bg-white/95 dark:bg-neutral-900/95 shadow-lg px-4 py-3 flex items-start gap-3">
          <div className="w-5 h-5 rounded-full bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 flex items-center justify-center shrink-0 mt-0.5">
            <Check className="w-3.5 h-3.5" />
          </div>
          <div className="min-w-0">
            <div className="text-[13px] font-medium text-neutral-900 dark:text-neutral-100">
              {toast.title}
            </div>
            <div className="text-[12px] font-mono text-neutral-500 dark:text-neutral-400 truncate mt-0.5">
              {toast.detail}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

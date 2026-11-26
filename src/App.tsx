import { useCallback, useEffect, useRef, useState } from 'react';
import { ActiveSlot, KeyRingConfig } from './types/keyring';
import { TRANSLATIONS } from './i18n/translations';
import { DEFAULT_CONFIG } from './data/presets';
import { PieMenuOverlay } from './components/PieMenuOverlay';
import { ConfigPage } from './components/ConfigPage';
import { TitleBar } from './components/TitleBar';
import {
  RuntimeStatus,
  emitNativeEvent,
  executeNativeSlot,
  fetchNativeConfig,
  fetchRuntimeStatus,
  getWindowMode,
  hidePieWindow,
  listenNativeEvent,
  persistNativeConfig,
} from './tauriBridge';

const CONFIG_STORAGE_KEY = 'keyring_config_v1';
const THEME_STORAGE_KEY = 'keyring_theme_v1';

function mergeNativeConfig(prev: KeyRingConfig, native: Partial<KeyRingConfig>): KeyRingConfig {
  return {
    ...DEFAULT_CONFIG,
    ...prev,
    ...native,
    trigger: { ...DEFAULT_CONFIG.trigger, ...prev.trigger, ...native.trigger },
    ui: { ...DEFAULT_CONFIG.ui, ...prev.ui, ...native.ui },
    activeSlots: Array.isArray(native.activeSlots) ? native.activeSlots : prev.activeSlots,
    libraryApps: Array.isArray(native.libraryApps) ? native.libraryApps : prev.libraryApps,
  };
}

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
  const [darkMode, setDarkMode] = useState<boolean>(() => {
    try {
      const saved = localStorage.getItem(THEME_STORAGE_KEY);
      if (saved === 'dark') return true;
      if (saved === 'light') return false;
      return loadInitialConfig().theme === 'dark';
    } catch {
      return false;
    }
  });
  const [pieVisible, setPieVisible] = useState(() => windowMode === 'pie');
  const [status, setStatus] = useState<RuntimeStatus | null>(null);
  const lastSyncedConfigJsonRef = useRef<string | null>(null);

  const t = TRANSLATIONS[config.language] || TRANSLATIONS['zh-CN'];

  useEffect(() => {
    if (windowMode === 'pie') {
      return;
    }
    const serialized = JSON.stringify(config);
    try {
      localStorage.setItem(CONFIG_STORAGE_KEY, serialized);
    } catch {
      // ignore quota errors
    }
    if (lastSyncedConfigJsonRef.current === serialized) {
      return;
    }
    lastSyncedConfigJsonRef.current = serialized;
    void persistNativeConfig(config);
  }, [config, windowMode]);

  useEffect(() => {
    document.documentElement.classList.toggle('dark', darkMode);
    if (windowMode === 'pie') {
      return;
    }
    try {
      localStorage.setItem(THEME_STORAGE_KEY, darkMode ? 'dark' : 'light');
    } catch {
      // ignore
    }
  }, [darkMode, windowMode]);

  useEffect(() => {
    let cancelled = false;

    const refreshRuntime = () => {
      void fetchRuntimeStatus().then((runtime) => {
        if (!cancelled && runtime) {
          setStatus(runtime);
        }
      });
    };

    const applyNativeConfig = (native: KeyRingConfig | null | undefined) => {
      if (cancelled || !native || !Array.isArray(native.activeSlots)) {
        return;
      }
      setConfig((prev) => {
        const merged = mergeNativeConfig(prev, native);
        lastSyncedConfigJsonRef.current = JSON.stringify(merged);
        return merged;
      });
      if (native.theme === 'dark' || native.theme === 'light') {
        setDarkMode(native.theme === 'dark');
      }
    };

    void (async () => {
      const native = await fetchNativeConfig<KeyRingConfig>();
      if (native && Array.isArray(native.activeSlots) && native.activeSlots.length > 0) {
        applyNativeConfig(native);
      }
      refreshRuntime();
    })();

    const timer = window.setInterval(refreshRuntime, 1500);
    window.addEventListener('focus', refreshRuntime);

    const handleStorage = (e: StorageEvent) => {
      if (e.key === THEME_STORAGE_KEY && e.newValue) {
        setDarkMode(e.newValue === 'dark');
      }
    };
    window.addEventListener('storage', handleStorage);

    let unlistenPlacement: (() => void) | undefined;
    let unlistenStatus: (() => void) | undefined;
    let unlistenConfig: (() => void) | undefined;
    let unlistenTheme: (() => void) | undefined;

    void listenNativeEvent('placement-status', refreshRuntime).then((fn) => {
      unlistenPlacement = fn;
    });
    void listenNativeEvent('runtime-status-changed', refreshRuntime).then((fn) => {
      unlistenStatus = fn;
    });
    void listenNativeEvent<KeyRingConfig>('config-updated', (nextCfg) => {
      applyNativeConfig(nextCfg);
    }).then((fn) => {
      unlistenConfig = fn;
    });
    void listenNativeEvent<{ darkMode?: boolean }>('theme-changed', (payload) => {
      if (typeof payload?.darkMode === 'boolean') {
        setDarkMode(payload.darkMode);
      }
    }).then((fn) => {
      unlistenTheme = fn;
    });

    return () => {
      cancelled = true;
      window.clearInterval(timer);
      window.removeEventListener('focus', refreshRuntime);
      window.removeEventListener('storage', handleStorage);
      unlistenPlacement?.();
      unlistenStatus?.();
      unlistenConfig?.();
      unlistenTheme?.();
    };
  }, []);

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

  useEffect(() => {
    if (windowMode !== 'pie') {
      return;
    }
    let unlisten: (() => void) | undefined;
    void listenNativeEvent('pie-shown', () => {
      setPieVisible(true);
      try {
        const storedTheme = localStorage.getItem(THEME_STORAGE_KEY);
        if (storedTheme === 'dark' || storedTheme === 'light') {
          setDarkMode(storedTheme === 'dark');
        }
      } catch {
        // ignore
      }
      void fetchNativeConfig<KeyRingConfig>().then((native) => {
        if (native && Array.isArray(native.activeSlots)) {
          setConfig((prev) => mergeNativeConfig(prev, native));
          if (native.theme === 'dark' || native.theme === 'light') {
            setDarkMode(native.theme === 'dark');
          }
        }
      });
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, [windowMode]);

  const handleToggleDarkMode = useCallback(() => {
    setDarkMode((prev) => {
      const next = !prev;
      const nextTheme = next ? 'dark' : 'light';
      try {
        localStorage.setItem(THEME_STORAGE_KEY, nextTheme);
      } catch {
        // ignore
      }
      setConfig((cfg) => ({ ...cfg, theme: nextTheme }));
      void emitNativeEvent('theme-changed', { darkMode: next });
      return next;
    });
  }, []);

  const closePieMenu = useCallback(() => {
    setPieVisible(false);
    if (windowMode === 'pie') {
      void hidePieWindow();
    }
  }, [windowMode]);

  const handleExecuteSlot = useCallback(
    async (slot: ActiveSlot) => {
      if (windowMode === 'pie') {
        await executeNativeSlot(slot.id);
        await hidePieWindow();
        setPieVisible(false);
        return;
      }
      setPieVisible(false);
    },
    [windowMode],
  );

  if (windowMode === 'pie') {
    const center = {
      x: typeof window !== 'undefined' ? Math.round(window.innerWidth / 2) : 170,
      y: typeof window !== 'undefined' ? Math.round(window.innerHeight / 2) : 170,
    };
    return (
      <div className="w-screen h-screen bg-transparent overflow-hidden">
        <PieMenuOverlay
          visible={pieVisible}
          coords={center}
          slots={config.activeSlots}
          ringDiameter={config.ui.ringDiameter}
          innerHoleDiameter={config.ui.innerHoleDiameter}
          iconSize={config.ui.iconSize}
          t={t}
          darkSurface={darkMode}
          nativePieMode
          onExecute={handleExecuteSlot}
          onClose={closePieMenu}
        />
      </div>
    );
  }

  return (
    <div className="h-screen w-screen flex flex-col bg-white dark:bg-[#1F1F1F] text-[#1F1F1F] dark:text-[#EDEDEC] overflow-hidden">
      <TitleBar platform={status?.platform} />
      <ConfigPage
        config={config}
        t={t}
        darkMode={darkMode}
        status={status}
        onStatusChange={setStatus}
        onUpdateConfig={setConfig}
        onToggleDarkMode={handleToggleDarkMode}
      />
    </div>
  );
}

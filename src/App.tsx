import { useCallback, useEffect, useState } from 'react';
import { ActiveSlot, KeyRingConfig } from './types/keyring';
import { TRANSLATIONS } from './i18n/translations';
import { DEFAULT_CONFIG } from './data/presets';
import { PieMenuOverlay } from './components/PieMenuOverlay';
import { ConfigPage } from './components/ConfigPage';
import { TitleBar } from './components/TitleBar';
import {
  RuntimeStatus,
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
      return localStorage.getItem(THEME_STORAGE_KEY) === 'dark';
    } catch {
      return false;
    }
  });
  const [pieVisible, setPieVisible] = useState(() => windowMode === 'pie');
  const [status, setStatus] = useState<RuntimeStatus | null>(null);

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
    let cancelled = false;
    void (async () => {
      const native = await fetchNativeConfig<KeyRingConfig>();
      if (!cancelled && native && Array.isArray(native.activeSlots) && native.activeSlots.length > 0) {
        setConfig((prev) => ({
          ...DEFAULT_CONFIG,
          ...prev,
          ...native,
          trigger: { ...DEFAULT_CONFIG.trigger, ...native.trigger },
          ui: { ...DEFAULT_CONFIG.ui, ...native.ui },
        }));
      }
      const runtime = await fetchRuntimeStatus();
      if (!cancelled) {
        setStatus(runtime);
      }
    })();
    const timer = window.setInterval(() => {
      void fetchRuntimeStatus().then((runtime) => {
        if (runtime) setStatus(runtime);
      });
    }, 2500);
    let unlistenPlacement: (() => void) | undefined;
    void listenNativeEvent('placement-status', () => {
      void fetchRuntimeStatus().then((runtime) => {
        if (!cancelled && runtime) setStatus(runtime);
      });
    }).then((fn) => {
      unlistenPlacement = fn;
    });
    return () => {
      cancelled = true;
      window.clearInterval(timer);
      unlistenPlacement?.();
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
      void fetchNativeConfig<KeyRingConfig>().then((native) => {
        if (native && Array.isArray(native.activeSlots)) {
          setConfig((prev) => ({
            ...prev,
            ...native,
            trigger: { ...prev.trigger, ...native.trigger },
            ui: { ...prev.ui, ...native.ui },
            activeSlots: native.activeSlots,
          }));
        }
      });
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, [windowMode]);

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
      <TitleBar title="KeyRing" platform={status?.platform} />
      <ConfigPage
        config={config}
        t={t}
        darkMode={darkMode}
        status={status}
        onStatusChange={setStatus}
        onUpdateConfig={setConfig}
        onToggleDarkMode={() => setDarkMode((value) => !value)}
      />
    </div>
  );
}

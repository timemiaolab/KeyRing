import React, { useRef, useState } from 'react';
import { Copy, Check, Download, Upload, RotateCcw } from 'lucide-react';
import { KeyRingConfig, ModifierKey, SupportedLocale } from '../types/keyring';
import { LOCALE_OPTIONS, TranslationDict } from '../i18n/translations';
import { StateMachinePhase } from '../hooks/useDoubleTapTrigger';

interface PreferencesPageProps {
  config: KeyRingConfig;
  t: TranslationDict;
  triggerPhase: StateMachinePhase;
  lastEventNote: string;
  lastDeltaMs: number | null;
  onUpdateConfig: (updater: (prev: KeyRingConfig) => KeyRingConfig) => void;
  onResetDefaults: () => void;
  onImportConfig: (imported: KeyRingConfig) => void;
}

export const PreferencesPage: React.FC<PreferencesPageProps> = ({
  config,
  t,
  triggerPhase,
  lastEventNote,
  lastDeltaMs,
  onUpdateConfig,
  onResetDefaults,
  onImportConfig,
}) => {
  const [copiedJson, setCopiedJson] = useState(false);
  const [importError, setImportError] = useState('');
  const importInputRef = useRef<HTMLInputElement>(null);

  const modifiers: { id: ModifierKey; label: string; platformHint: string }[] = [
    { id: 'Command', label: 'Command (⌘)', platformHint: 'macOS' },
    { id: 'Control', label: 'Control (Ctrl)', platformHint: 'Windows / Linux' },
    { id: 'Alt', label: 'Alt / Option', platformHint: 'Windows / Linux / macOS' },
  ];

  const jsonPreview = JSON.stringify(config, null, 2);

  const handleCopyJson = async () => {
    try {
      await navigator.clipboard.writeText(jsonPreview);
      setCopiedJson(true);
      setTimeout(() => setCopiedJson(false), 1800);
    } catch {
      // Clipboard fallback ignored
    }
  };

  const handleDownloadJson = () => {
    const blob = new Blob([jsonPreview], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'config.json';
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleImportFile = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      try {
        const parsed = JSON.parse(String(reader.result));
        if (parsed && parsed.trigger && parsed.ui && Array.isArray(parsed.activeSlots)) {
          onImportConfig(parsed as KeyRingConfig);
          setImportError('');
        } else {
          setImportError('Invalid config.json structure');
        }
      } catch {
        setImportError('Failed to parse JSON file');
      }
    };
    reader.readAsText(file);
    e.target.value = '';
  };

  const phaseLabelMap: Record<StateMachinePhase, string> = {
    idle: t.preferences.stateIdle,
    holding: t.preferences.stateHolding,
    awaiting_second: t.preferences.stateAwaiting,
    contaminated: t.preferences.stateContaminated,
    triggered: t.preferences.stateTriggered,
  };

  const phaseTextColor: Record<StateMachinePhase, string> = {
    idle: 'text-neutral-600 dark:text-neutral-300',
    holding: 'text-[#2383E2]',
    awaiting_second: 'text-amber-600 dark:text-amber-400',
    contaminated: 'text-red-600 dark:text-red-400',
    triggered: 'text-emerald-600 dark:text-emerald-400',
  };

  return (
    <div className="max-w-4xl mx-auto px-6 md:px-14 py-10 md:py-12">
      <header className="pb-8 border-b border-neutral-200/80 dark:border-neutral-800">
        <h1 className="text-[28px] font-semibold tracking-tight text-neutral-900 dark:text-neutral-100 leading-tight">
          {t.preferences.title}
        </h1>
        <p className="mt-2 text-[14px] text-neutral-500 dark:text-neutral-400 leading-relaxed max-w-2xl">
          {t.preferences.subtitle}
        </p>
      </header>

      <div className="divide-y divide-neutral-200/80 dark:divide-neutral-800">
        {/* 1. Language Switcher (10 Languages) */}
        <section className="py-8 grid grid-cols-1 md:grid-cols-12 gap-6 items-start">
          <div className="md:col-span-5">
            <h2 className="text-[15px] font-semibold text-neutral-900 dark:text-neutral-100">
              {t.preferences.languageSection}
            </h2>
            <p className="text-[13px] text-neutral-500 dark:text-neutral-400 mt-1 leading-relaxed">
              {t.preferences.languageDesc}
            </p>
          </div>
          <div className="md:col-span-7">
            <label
              htmlFor="locale-select"
              className="block text-[12.5px] font-medium text-neutral-700 dark:text-neutral-300 mb-2"
            >
              {t.preferences.languageLabel}
            </label>
            <select
              id="locale-select"
              value={config.language}
              onChange={(e) =>
                onUpdateConfig((prev) => ({
                  ...prev,
                  language: e.target.value as SupportedLocale,
                }))
              }
              className="w-full max-w-sm px-3 py-2 rounded-lg text-[13.5px] border border-neutral-200 dark:border-neutral-700 bg-[#FAFAF9] dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100 focus:outline-none focus:border-[#2383E2]"
            >
              {LOCALE_OPTIONS.map((loc) => (
                <option key={loc.code} value={loc.code}>
                  {loc.nativeName} · {loc.code} ({loc.regionHint})
                </option>
              ))}
            </select>
          </div>
        </section>

        {/* 2. Global Hotkey & Anti-Mistouch State Machine */}
        <section className="py-8 grid grid-cols-1 md:grid-cols-12 gap-6 items-start">
          <div className="md:col-span-5">
            <h2 className="text-[15px] font-semibold text-neutral-900 dark:text-neutral-100">
              {t.preferences.triggerSection}
            </h2>
            <p className="text-[13px] text-neutral-500 dark:text-neutral-400 mt-1 leading-relaxed">
              {t.preferences.modifierDesc}
            </p>
          </div>

          <div className="md:col-span-7 space-y-6">
            <div>
              <div className="text-[12.5px] font-medium text-neutral-700 dark:text-neutral-300 mb-2">
                {t.preferences.modifierLabel}
              </div>
              <div className="inline-flex flex-wrap gap-1 p-1 rounded-lg bg-[#F3F3F1] dark:bg-neutral-800/80">
                {modifiers.map((m) => {
                  const active = config.trigger.modifier === m.id;
                  return (
                    <button
                      key={m.id}
                      onClick={() =>
                        onUpdateConfig((prev) => ({
                          ...prev,
                          trigger: { ...prev.trigger, modifier: m.id },
                        }))
                      }
                      className={`px-3 py-1.5 rounded-md text-[12.5px] font-medium transition-colors whitespace-nowrap ${
                        active
                          ? 'bg-white dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100 shadow-xs'
                          : 'text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'
                      }`}
                    >
                      {m.label}
                    </button>
                  );
                })}
              </div>
            </div>

            <div>
              <div className="flex items-center justify-between mb-1.5">
                <label
                  htmlFor="interval-slider"
                  className="text-[12.5px] font-medium text-neutral-700 dark:text-neutral-300"
                >
                  {t.preferences.intervalLabel}
                </label>
                <span className="text-[13px] font-mono tabular-nums font-medium text-neutral-900 dark:text-neutral-100">
                  {config.trigger.intervalMs} ms
                </span>
              </div>
              <input
                id="interval-slider"
                type="range"
                min={150}
                max={500}
                step={10}
                value={config.trigger.intervalMs}
                onChange={(e) =>
                  onUpdateConfig((prev) => ({
                    ...prev,
                    trigger: { ...prev.trigger, intervalMs: Number(e.target.value) },
                  }))
                }
                className="w-full accent-[#2383E2] cursor-pointer"
              />
              <p className="text-[12px] text-neutral-500 dark:text-neutral-400 mt-1">
                {t.preferences.intervalDesc}
              </p>
            </div>

            {/* Live State Machine Inspector */}
            <div className="rounded-xl border border-neutral-200/80 dark:border-neutral-800 bg-[#FAFAF9] dark:bg-[#171717] p-4">
              <div className="flex items-center justify-between gap-2">
                <span className="text-[13px] font-medium text-neutral-800 dark:text-neutral-200">
                  {t.preferences.antiMistouchTitle}
                </span>
                {lastDeltaMs !== null && (
                  <span className="text-[12px] font-mono tabular-nums text-neutral-500">
                    Δ {lastDeltaMs} ms
                  </span>
                )}
              </div>
              <p className="text-[12px] text-neutral-500 dark:text-neutral-400 mt-1">
                {t.preferences.antiMistouchDesc}
              </p>
              <div className="mt-3 pt-3 border-t border-neutral-200/70 dark:border-neutral-800 flex items-center justify-between text-[12.5px]">
                <span className={`font-medium ${phaseTextColor[triggerPhase]}`}>
                  {phaseLabelMap[triggerPhase]}
                </span>
                <span className="font-mono text-[11.5px] text-neutral-400 truncate max-w-[220px]">
                  {lastEventNote || `${config.trigger.modifier} · ${config.trigger.intervalMs}ms`}
                </span>
              </div>
            </div>
          </div>
        </section>

        {/* 3. Appearance & System */}
        <section className="py-8 grid grid-cols-1 md:grid-cols-12 gap-6 items-start">
          <div className="md:col-span-5">
            <h2 className="text-[15px] font-semibold text-neutral-900 dark:text-neutral-100">
              {t.preferences.appearanceSection}
            </h2>
          </div>

          <div className="md:col-span-7 space-y-5">
            <div>
              <div className="flex items-center justify-between mb-1">
                <label className="text-[12.5px] font-medium text-neutral-700 dark:text-neutral-300">
                  {t.preferences.ringDiameterLabel}
                </label>
                <span className="text-[13px] font-mono tabular-nums text-neutral-800 dark:text-neutral-200">
                  {config.ui.ringDiameter} px
                </span>
              </div>
              <input
                type="range"
                min={200}
                max={340}
                step={4}
                value={config.ui.ringDiameter}
                onChange={(e) =>
                  onUpdateConfig((prev) => ({
                    ...prev,
                    ui: { ...prev.ui, ringDiameter: Number(e.target.value) },
                  }))
                }
                className="w-full accent-[#2383E2] cursor-pointer"
              />
            </div>

            <div>
              <div className="flex items-center justify-between mb-1">
                <label className="text-[12.5px] font-medium text-neutral-700 dark:text-neutral-300">
                  {t.preferences.innerHoleLabel}
                </label>
                <span className="text-[13px] font-mono tabular-nums text-neutral-800 dark:text-neutral-200">
                  {config.ui.innerHoleDiameter} px
                </span>
              </div>
              <input
                type="range"
                min={60}
                max={120}
                step={2}
                value={config.ui.innerHoleDiameter}
                onChange={(e) =>
                  onUpdateConfig((prev) => ({
                    ...prev,
                    ui: { ...prev.ui, innerHoleDiameter: Number(e.target.value) },
                  }))
                }
                className="w-full accent-[#2383E2] cursor-pointer"
              />
            </div>

            <div>
              <div className="flex items-center justify-between mb-1">
                <label className="text-[12.5px] font-medium text-neutral-700 dark:text-neutral-300">
                  {t.preferences.iconSizeLabel}
                </label>
                <span className="text-[13px] font-mono tabular-nums text-neutral-800 dark:text-neutral-200">
                  {config.ui.iconSize} px
                </span>
              </div>
              <input
                type="range"
                min={24}
                max={48}
                step={2}
                value={config.ui.iconSize}
                onChange={(e) =>
                  onUpdateConfig((prev) => ({
                    ...prev,
                    ui: { ...prev.ui, iconSize: Number(e.target.value) },
                  }))
                }
                className="w-full accent-[#2383E2] cursor-pointer"
              />
            </div>

            {/* Autostart Toggle */}
            <div className="pt-2 flex items-center justify-between gap-4">
              <div>
                <div className="text-[13.5px] font-medium text-neutral-900 dark:text-neutral-100">
                  {t.preferences.autostartLabel}
                </div>
                <div className="text-[12.5px] text-neutral-500 dark:text-neutral-400">
                  {t.preferences.autostartDesc}
                </div>
              </div>
              <button
                type="button"
                role="switch"
                aria-checked={config.autostart}
                onClick={() =>
                  onUpdateConfig((prev) => ({ ...prev, autostart: !prev.autostart }))
                }
                className={`relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full transition-colors duration-150 ${
                  config.autostart ? 'bg-[#2383E2]' : 'bg-neutral-300 dark:bg-neutral-700'
                }`}
              >
                <span
                  className={`inline-block h-4 w-4 transform rounded-full bg-white shadow transition-transform duration-150 mt-0.5 ${
                    config.autostart ? 'translate-x-4.5' : 'translate-x-0.5'
                  }`}
                />
              </button>
            </div>
          </div>
        </section>

        {/* 4. config.json Persistence View */}
        <section className="py-8 grid grid-cols-1 md:grid-cols-12 gap-6 items-start">
          <div className="md:col-span-5">
            <h2 className="text-[15px] font-semibold text-neutral-900 dark:text-neutral-100">
              {t.preferences.configSection}
            </h2>
            <p className="text-[13px] text-neutral-500 dark:text-neutral-400 mt-1 leading-relaxed">
              {t.preferences.configDesc}
            </p>
            <div className="mt-4 flex flex-wrap gap-2">
              <button
                onClick={handleCopyJson}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-[12.5px] border border-neutral-200 dark:border-neutral-700 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 whitespace-nowrap"
              >
                {copiedJson ? <Check className="w-3.5 h-3.5 text-emerald-600" /> : <Copy className="w-3.5 h-3.5" />}
                <span>{copiedJson ? t.preferences.configCopied : t.preferences.copyConfig}</span>
              </button>

              <button
                onClick={handleDownloadJson}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-[12.5px] border border-neutral-200 dark:border-neutral-700 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 whitespace-nowrap"
              >
                <Download className="w-3.5 h-3.5" />
                <span>{t.preferences.downloadConfig}</span>
              </button>

              <input
                ref={importInputRef}
                type="file"
                accept=".json,application/json"
                onChange={handleImportFile}
                className="hidden"
              />
              <button
                onClick={() => importInputRef.current?.click()}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-[12.5px] border border-neutral-200 dark:border-neutral-700 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 whitespace-nowrap"
              >
                <Upload className="w-3.5 h-3.5" />
                <span>{t.preferences.importConfig}</span>
              </button>

              <button
                onClick={onResetDefaults}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-[12.5px] text-neutral-500 hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/30 whitespace-nowrap"
              >
                <RotateCcw className="w-3.5 h-3.5" />
                <span>{t.preferences.resetConfig}</span>
              </button>
            </div>
            {importError && (
              <p className="mt-2 text-[12px] text-red-600 dark:text-red-400">{importError}</p>
            )}
          </div>

          <div className="md:col-span-7">
            <pre className="rounded-xl border border-neutral-200/80 dark:border-neutral-800 bg-[#FAFAF9] dark:bg-[#171717] p-4 text-[12px] font-mono text-neutral-700 dark:text-neutral-300 overflow-x-auto max-h-80 leading-relaxed">
              {jsonPreview}
            </pre>
          </div>
        </section>
      </div>
    </div>
  );
};

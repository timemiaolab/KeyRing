import React, { useRef, useState } from 'react';
import { Plus, Check, Trash2, FolderOpen, X, Sparkles } from 'lucide-react';
import {
  BuiltinAction,
  KeyRingConfig,
  LibraryApp,
  TerminalAdapter,
} from '../types/keyring';
import { TranslationDict } from '../i18n/translations';
import { COMMON_IDE_PRESETS, TERMINAL_ADAPTER_OPTIONS } from '../data/presets';
import { AppBrandIcon } from './AppBrandIcon';

interface LibraryPageProps {
  config: KeyRingConfig;
  t: TranslationDict;
  onAddBuiltinToRing: (action: BuiltinAction) => void;
  onRemoveSlot: (slotId: string) => void;
  onAddAppToRing: (app: LibraryApp) => void;
  onAddAppToLibrary: (app: LibraryApp) => void;
  onDeleteLibraryApp: (appId: string) => void;
  onChangeTerminalApp: (adapter: TerminalAdapter) => void;
  onImportMissingIdes: (apps: LibraryApp[]) => void;
}

export const LibraryPage: React.FC<LibraryPageProps> = ({
  config,
  t,
  onAddBuiltinToRing,
  onRemoveSlot,
  onAddAppToRing,
  onAddAppToLibrary,
  onDeleteLibraryApp,
  onChangeTerminalApp,
  onImportMissingIdes,
}) => {
  const [modalOpen, setModalOpen] = useState(false);
  const [customName, setCustomName] = useState('');
  const [customPath, setCustomPath] = useState('');
  const [customIconUrl, setCustomIconUrl] = useState<string | undefined>(undefined);
  const [ringFullNotice, setRingFullNotice] = useState(false);

  const fileInputRef = useRef<HTMLInputElement>(null);
  const iconInputRef = useRef<HTMLInputElement>(null);

  const isRingFull = config.activeSlots.length >= 8;

  const isBuiltinActive = (action: BuiltinAction) =>
    config.activeSlots.some((s) => s.type === 'builtin' && s.action === action);

  const getBuiltinSlotId = (action: BuiltinAction) =>
    config.activeSlots.find((s) => s.type === 'builtin' && s.action === action)?.id;

  const isAppActiveOnRing = (appId: string, execPath: string) =>
    config.activeSlots.some(
      (s) => s.type === 'app' && (s.id === appId || s.execPath === execPath)
    );

  const isAppInLibrary = (appId: string, execPath: string) =>
    config.libraryApps.some((a) => a.id === appId || a.execPath === execPath);

  const handleTryAddBuiltin = (action: BuiltinAction) => {
    if (isRingFull) {
      setRingFullNotice(true);
      setTimeout(() => setRingFullNotice(false), 2600);
      return;
    }
    onAddBuiltinToRing(action);
  };

  const handleTryAddApp = (app: LibraryApp) => {
    if (isRingFull) {
      setRingFullNotice(true);
      setTimeout(() => setRingFullNotice(false), 2600);
      return;
    }
    onAddAppToRing(app);
  };

  const handlePickExecutable = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const rawName = file.name.replace(/\.(app|exe|desktop|AppImage)$/i, '');
    if (!customName) setCustomName(rawName);
    setCustomPath(`/Applications/${file.name}`);
  };

  const handlePickIcon = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      if (typeof reader.result === 'string') {
        setCustomIconUrl(reader.result);
      }
    };
    reader.readAsDataURL(file);
  };

  const handleCreateCustomApp = (e: React.FormEvent) => {
    e.preventDefault();
    const trimmedName = customName.trim();
    const trimmedPath = customPath.trim();
    if (!trimmedName || !trimmedPath) return;

    const newApp: LibraryApp = {
      id: `app-custom-${Date.now()}`,
      name: trimmedName,
      execPath: trimmedPath,
      iconCache: `icons/${trimmedName.toLowerCase().replace(/\s+/g, '-')}.png`,
      customIconDataUrl: customIconUrl,
      category: 'custom',
    };

    onAddAppToLibrary(newApp);
    setCustomName('');
    setCustomPath('');
    setCustomIconUrl(undefined);
    setModalOpen(false);
  };

  const missingPresets = COMMON_IDE_PRESETS.filter(
    (preset) => !isAppActiveOnRing(preset.id, preset.execPath) && !isAppInLibrary(preset.id, preset.execPath)
  );

  const builtins: BuiltinAction[] = ['copy_path', 'open_in_terminal'];

  return (
    <div className="max-w-4xl mx-auto px-6 md:px-14 py-10 md:py-12">
      {/* Header */}
      <header className="pb-8 border-b border-neutral-200/80 dark:border-neutral-800">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div className="max-w-2xl">
            <h1 className="text-[28px] font-semibold tracking-tight text-neutral-900 dark:text-neutral-100 leading-tight">
              {t.library.title}
            </h1>
            <p className="mt-2 text-[14px] text-neutral-500 dark:text-neutral-400 leading-relaxed">
              {t.library.subtitle}
            </p>
          </div>
          <button
            onClick={() => setModalOpen(true)}
            className="inline-flex items-center gap-2 px-4 py-2 rounded-lg text-[13px] font-medium bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900 hover:bg-neutral-800 dark:hover:bg-neutral-200 transition-colors whitespace-nowrap shrink-0"
          >
            <Plus className="w-4 h-4" />
            <span>{t.library.addCustomApp}</span>
          </button>
        </div>

        {ringFullNotice && (
          <div className="mt-4 px-3.5 py-2 rounded-lg bg-amber-50 dark:bg-amber-950/40 border border-amber-200 dark:border-amber-800/60 text-[13px] text-amber-800 dark:text-amber-300">
            {t.library.ringFullError}
          </div>
        )}
      </header>

      {/* Section 1: Built-in Default Actions */}
      <section className="mt-10">
        <div className="mb-4">
          <h2 className="text-[16px] font-semibold text-neutral-900 dark:text-neutral-100">
            {t.library.builtinSectionTitle}
          </h2>
          <p className="text-[13px] text-neutral-500 dark:text-neutral-400 mt-0.5">
            {t.library.builtinSectionDesc}
          </p>
        </div>

        <div className="divide-y divide-neutral-200/70 dark:divide-neutral-800 border border-neutral-200/80 dark:border-neutral-800 rounded-xl bg-white dark:bg-[#1C1C1C]">
          {builtins.map((action) => {
            const active = isBuiltinActive(action);
            const activeId = getBuiltinSlotId(action);
            const title = t.builtins[action];
            const desc =
              action === 'copy_path'
                ? t.builtins.copy_path_desc
                : t.builtins.open_in_terminal_desc;

            return (
              <div
                key={action}
                className="p-4 flex flex-col sm:flex-row sm:items-center justify-between gap-4"
              >
                <div className="flex items-start gap-3.5 min-w-0">
                  <AppBrandIcon builtinAction={action} size={36} />
                  <div className="min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="text-[14px] font-medium text-neutral-900 dark:text-neutral-100">
                        {title}
                      </span>
                      <span className="text-[12px] font-mono text-neutral-400 dark:text-neutral-500">
                        · {action}
                      </span>
                    </div>
                    <p className="text-[12.5px] text-neutral-500 dark:text-neutral-400 mt-0.5 leading-relaxed">
                      {desc}
                    </p>

                    {action === 'open_in_terminal' && (
                      <div className="mt-2.5 flex flex-wrap items-center gap-2">
                        <label
                          htmlFor="terminal-adapter-select"
                          className="text-[12px] text-neutral-500 dark:text-neutral-400"
                        >
                          {t.library.terminalPreferenceLabel}:
                        </label>
                        <select
                          id="terminal-adapter-select"
                          value={config.terminalApp}
                          onChange={(e) => onChangeTerminalApp(e.target.value as TerminalAdapter)}
                          className="text-[12px] px-2.5 py-1 rounded-md border border-neutral-200 dark:border-neutral-700 bg-[#FAFAF9] dark:bg-neutral-800 text-neutral-800 dark:text-neutral-200 focus:outline-none focus:border-[#2383E2]"
                        >
                          {TERMINAL_ADAPTER_OPTIONS.map((opt) => (
                            <option key={opt.id} value={opt.id}>
                              {opt.label} ({opt.platform})
                            </option>
                          ))}
                        </select>
                      </div>
                    )}
                  </div>
                </div>

                <div className="flex items-center gap-2 shrink-0 self-end sm:self-center">
                  {active ? (
                    <>
                      <span className="inline-flex items-center gap-1 text-[12.5px] font-medium text-[#2383E2] mr-2 whitespace-nowrap">
                        <Check className="w-3.5 h-3.5" />
                        {t.library.addedBadge}
                      </span>
                      <button
                        onClick={() => activeId && onRemoveSlot(activeId)}
                        className="px-3 py-1.5 rounded-lg text-[12.5px] border border-neutral-200 dark:border-neutral-700 text-neutral-600 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors whitespace-nowrap"
                      >
                        {t.library.removeFromRing}
                      </button>
                    </>
                  ) : (
                    <button
                      onClick={() => handleTryAddBuiltin(action)}
                      className="px-3.5 py-1.5 rounded-lg text-[12.5px] font-medium bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900 hover:opacity-90 transition-opacity whitespace-nowrap"
                    >
                      {t.library.addToRing}
                    </button>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </section>

      {/* Section 2: Library External Apps (Saved but currently off-ring) */}
      <section className="mt-10">
        <div className="mb-4">
          <h2 className="text-[16px] font-semibold text-neutral-900 dark:text-neutral-100">
            {t.library.customSectionTitle}
          </h2>
          <p className="text-[13px] text-neutral-500 dark:text-neutral-400 mt-0.5">
            {t.library.customSectionDesc}
          </p>
        </div>

        {config.libraryApps.length === 0 ? (
          <div className="rounded-xl border border-neutral-200/80 dark:border-neutral-800 bg-[#FAFAF9] dark:bg-[#171717] px-5 py-6 text-[13px] text-neutral-500 dark:text-neutral-400">
            {t.library.noCustomApps}
          </div>
        ) : (
          <div className="divide-y divide-neutral-200/70 dark:divide-neutral-800 border border-neutral-200/80 dark:border-neutral-800 rounded-xl bg-white dark:bg-[#1C1C1C]">
            {config.libraryApps.map((app) => (
              <div key={app.id} className="px-4 py-3 flex items-center justify-between gap-4">
                <div className="flex items-center gap-3 min-w-0">
                  <AppBrandIcon
                    presetKey={app.presetKey}
                    customIconDataUrl={app.customIconDataUrl}
                    name={app.name}
                    size={32}
                  />
                  <div className="min-w-0">
                    <div className="text-[13.5px] font-medium text-neutral-900 dark:text-neutral-100 truncate">
                      {app.name}
                    </div>
                    <div className="text-[12px] font-mono text-neutral-500 dark:text-neutral-400 truncate">
                      {app.execPath}
                    </div>
                  </div>
                </div>

                <div className="flex items-center gap-2 shrink-0">
                  <button
                    onClick={() => handleTryAddApp(app)}
                    className="px-3 py-1.5 rounded-lg text-[12.5px] font-medium bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900 hover:opacity-90 transition-opacity whitespace-nowrap"
                  >
                    {t.library.addToRing}
                  </button>
                  <button
                    onClick={() => onDeleteLibraryApp(app.id)}
                    title={t.library.deleteApp}
                    className="p-1.5 rounded-lg text-neutral-400 hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/40 transition-colors"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>

      {/* Section 3: Common IDE One-Click Import */}
      <section className="mt-10">
        <div className="flex flex-wrap items-center justify-between gap-3 mb-4">
          <div>
            <h2 className="text-[16px] font-semibold text-neutral-900 dark:text-neutral-100">
              {t.library.externalSectionTitle}
            </h2>
            <p className="text-[13px] text-neutral-500 dark:text-neutral-400 mt-0.5">
              {t.library.externalSectionDesc}
            </p>
          </div>
          {missingPresets.length > 0 && (
            <button
              onClick={() => onImportMissingIdes(missingPresets)}
              className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-[12.5px] font-medium border border-neutral-200 dark:border-neutral-700 text-neutral-700 dark:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors whitespace-nowrap"
            >
              <Sparkles className="w-3.5 h-3.5 text-[#2383E2]" />
              <span>{t.library.importAllMissing}</span>
            </button>
          )}
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          {COMMON_IDE_PRESETS.map((preset) => {
            const onRing = isAppActiveOnRing(preset.id, preset.execPath);
            const inLib = isAppInLibrary(preset.id, preset.execPath);

            return (
              <div
                key={preset.id}
                className="flex items-center justify-between gap-3 p-3.5 rounded-xl border border-neutral-200/80 dark:border-neutral-800 bg-white dark:bg-[#1C1C1C]"
              >
                <div className="flex items-center gap-3 min-w-0">
                  <AppBrandIcon presetKey={preset.presetKey} name={preset.name} size={32} />
                  <div className="min-w-0">
                    <div className="text-[13.5px] font-medium text-neutral-900 dark:text-neutral-100 truncate">
                      {preset.name}
                    </div>
                    <div className="text-[11.5px] font-mono text-neutral-400 dark:text-neutral-500 truncate">
                      {preset.execPath}
                    </div>
                  </div>
                </div>

                <div className="shrink-0">
                  {onRing ? (
                    <span className="text-[12px] font-medium text-[#2383E2] whitespace-nowrap">
                      {t.library.addedBadge}
                    </span>
                  ) : inLib ? (
                    <button
                      onClick={() => handleTryAddApp(preset)}
                      className="px-2.5 py-1 rounded-md text-[12px] font-medium border border-neutral-200 dark:border-neutral-700 text-neutral-700 dark:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800 whitespace-nowrap"
                    >
                      {t.library.addToRing}
                    </button>
                  ) : (
                    <button
                      onClick={() => handleTryAddApp(preset)}
                      className="px-2.5 py-1 rounded-md text-[12px] font-medium bg-[#F7F7F5] dark:bg-neutral-800 text-neutral-800 dark:text-neutral-200 hover:bg-neutral-200/70 dark:hover:bg-neutral-700 transition-colors whitespace-nowrap"
                    >
                      + {t.library.addToRing}
                    </button>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </section>

      {/* Modal: Add Custom External Application */}
      {modalOpen && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/30 backdrop-blur-[1px] p-4"
          onMouseDown={(e) => {
            if (e.target === e.currentTarget) setModalOpen(false);
          }}
        >
          <div className="w-full max-w-md rounded-xl bg-white dark:bg-[#1E1E1E] border border-neutral-200 dark:border-neutral-700 shadow-xl p-6">
            <div className="flex items-center justify-between pb-4 border-b border-neutral-200/70 dark:border-neutral-800">
              <h3 className="text-[16px] font-semibold text-neutral-900 dark:text-neutral-100">
                {t.library.modalTitle}
              </h3>
              <button
                onClick={() => setModalOpen(false)}
                className="p-1 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-400"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <form onSubmit={handleCreateCustomApp} className="mt-4 space-y-4">
              <div>
                <label className="block text-[12.5px] font-medium text-neutral-700 dark:text-neutral-300 mb-1.5">
                  {t.library.modalNameLabel}
                </label>
                <input
                  type="text"
                  required
                  value={customName}
                  onChange={(e) => setCustomName(e.target.value)}
                  placeholder="Obsidian"
                  className="w-full px-3 py-2 rounded-lg text-[13.5px] border border-neutral-200 dark:border-neutral-700 bg-[#FAFAF9] dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100 focus:outline-none focus:border-[#2383E2]"
                />
              </div>

              <div>
                <label className="block text-[12.5px] font-medium text-neutral-700 dark:text-neutral-300 mb-1.5">
                  {t.library.modalPathLabel}
                </label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    required
                    value={customPath}
                    onChange={(e) => setCustomPath(e.target.value)}
                    placeholder={t.library.modalPathPlaceholder}
                    className="flex-1 px-3 py-2 rounded-lg text-[13px] font-mono border border-neutral-200 dark:border-neutral-700 bg-[#FAFAF9] dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100 focus:outline-none focus:border-[#2383E2]"
                  />
                  <input
                    ref={fileInputRef}
                    type="file"
                    onChange={handlePickExecutable}
                    className="hidden"
                  />
                  <button
                    type="button"
                    onClick={() => fileInputRef.current?.click()}
                    className="px-3 py-2 rounded-lg text-[12.5px] border border-neutral-200 dark:border-neutral-700 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 inline-flex items-center gap-1.5 shrink-0"
                  >
                    <FolderOpen className="w-3.5 h-3.5" />
                    <span>{t.library.modalPickFile}</span>
                  </button>
                </div>
              </div>

              <div>
                <label className="block text-[12.5px] font-medium text-neutral-700 dark:text-neutral-300 mb-1.5">
                  {t.library.modalIconLabel}
                </label>
                <div className="flex items-center gap-3">
                  <AppBrandIcon
                    customIconDataUrl={customIconUrl}
                    name={customName || 'A'}
                    size={36}
                  />
                  <input
                    ref={iconInputRef}
                    type="file"
                    accept="image/*"
                    onChange={handlePickIcon}
                    className="hidden"
                  />
                  <button
                    type="button"
                    onClick={() => iconInputRef.current?.click()}
                    className="px-3 py-1.5 rounded-lg text-[12.5px] border border-neutral-200 dark:border-neutral-700 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800"
                  >
                    {t.library.modalPickFile}
                  </button>
                </div>
              </div>

              <div className="pt-3 flex items-center justify-end gap-2 border-t border-neutral-200/70 dark:border-neutral-800">
                <button
                  type="button"
                  onClick={() => setModalOpen(false)}
                  className="px-4 py-2 rounded-lg text-[13px] text-neutral-600 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800"
                >
                  {t.library.modalCancel}
                </button>
                <button
                  type="submit"
                  className="px-4 py-2 rounded-lg text-[13px] font-medium bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900 hover:opacity-90"
                >
                  {t.library.modalConfirm}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};

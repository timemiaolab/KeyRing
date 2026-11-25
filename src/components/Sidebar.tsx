import React, { useState } from 'react';
import {
  Disc,
  Layers,
  SlidersHorizontal,
  TerminalSquare,
  Code2,
  Sun,
  Moon,
  Command,
  Pause,
  Play,
  Check,
  Power,
  Menu,
  X,
} from 'lucide-react';
import { ModifierKey, StudioPage } from '../types/keyring';
import { TranslationDict } from '../i18n/translations';

interface SidebarProps {
  activePage: StudioPage;
  onNavigate: (page: StudioPage) => void;
  t: TranslationDict;
  activeSlotCount: number;
  modifier: ModifierKey;
  intervalMs: number;
  autostart: boolean;
  onToggleAutostart: () => void;
  listenerPaused: boolean;
  onToggleListenerPaused: () => void;
  darkMode: boolean;
  onToggleDarkMode: () => void;
  onTestPopup: () => void;
  mobileOpen: boolean;
  onSetMobileOpen: (open: boolean) => void;
}

export const Sidebar: React.FC<SidebarProps> = ({
  activePage,
  onNavigate,
  t,
  activeSlotCount,
  modifier,
  intervalMs,
  autostart,
  onToggleAutostart,
  listenerPaused,
  onToggleListenerPaused,
  darkMode,
  onToggleDarkMode,
  onTestPopup,
  mobileOpen,
  onSetMobileOpen,
}) => {
  const [trayOpen, setTrayOpen] = useState(false);

  const navItems: { id: StudioPage; label: string; icon: React.ReactNode }[] = [
    { id: 'composer', label: t.nav.composer, icon: <Disc className="w-4 h-4" /> },
    { id: 'library', label: t.nav.library, icon: <Layers className="w-4 h-4" /> },
    { id: 'preferences', label: t.nav.preferences, icon: <SlidersHorizontal className="w-4 h-4" /> },
    { id: 'sandbox', label: t.nav.sandbox, icon: <TerminalSquare className="w-4 h-4" /> },
    { id: 'source', label: t.nav.source, icon: <Code2 className="w-4 h-4" /> },
  ];

  const sidebarContent = (
    <aside className="w-[248px] h-full flex flex-col justify-between bg-[#F7F7F5] dark:bg-[#191919] border-r border-[#EAEAE7] dark:border-neutral-800/90 select-none">
      {/* Top Workspace Identity (No OS System Titlebar) */}
      <div className="px-3 pt-5 pb-3">
        <div className="flex items-center justify-between px-2.5 py-1.5 mb-4">
          <div className="flex items-center gap-2.5 min-w-0">
            <div className="w-6 h-6 rounded-md bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900 flex items-center justify-center shrink-0">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
                <circle cx="12" cy="12" r="8" stroke="currentColor" strokeWidth="2.4" />
                <circle cx="12" cy="12" r="3" fill="currentColor" />
              </svg>
            </div>
            <div className="min-w-0">
              <div className="text-[14px] font-semibold text-neutral-900 dark:text-neutral-100 leading-tight truncate">
                {t.appName}
              </div>
              <div className="text-[11px] text-neutral-500 dark:text-neutral-400 truncate">
                {t.workspaceTagline}
              </div>
            </div>
          </div>
          <button
            onClick={() => onSetMobileOpen(false)}
            className="md:hidden p-1 rounded hover:bg-neutral-200/60 dark:hover:bg-neutral-800 text-neutral-500"
            aria-label="Close sidebar"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Navigation Section */}
        <div className="px-2.5 mb-1.5 text-[11px] font-medium text-neutral-400 dark:text-neutral-500">
          {t.sidebar.pagesGroup}
        </div>
        <nav className="space-y-0.5">
          {navItems.map((item) => {
            const isActive = activePage === item.id;
            return (
              <button
                key={item.id}
                onClick={() => {
                  onNavigate(item.id);
                  onSetMobileOpen(false);
                }}
                className={`w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-[13.5px] transition-colors duration-100 text-left whitespace-nowrap ${
                  isActive
                    ? 'bg-[#EAEAE6] dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 font-medium'
                    : 'text-neutral-600 dark:text-neutral-400 hover:bg-[#EFEFEC] dark:hover:bg-neutral-800/60 hover:text-neutral-900 dark:hover:text-neutral-200'
                }`}
              >
                <span className={isActive ? 'text-neutral-900 dark:text-neutral-100' : 'text-neutral-400 dark:text-neutral-500'}>
                  {item.icon}
                </span>
                <span className="truncate">{item.label}</span>
              </button>
            );
          })}
        </nav>
      </div>

      {/* Bottom Controls: Quick Trigger, Tray Menu Preview & Theme */}
      <div className="px-3 pb-4 pt-3 border-t border-[#EAEAE7] dark:border-neutral-800 space-y-2 relative">
        <div className="px-2.5 text-[11px] font-medium text-neutral-400 dark:text-neutral-500">
          {t.sidebar.runtimeGroup}
        </div>

        <button
          onClick={onTestPopup}
          className="w-full flex items-center justify-between px-2.5 py-2 rounded-md text-[13px] font-medium bg-white dark:bg-neutral-800/80 border border-neutral-200/80 dark:border-neutral-700/80 text-neutral-800 dark:text-neutral-100 hover:border-neutral-300 dark:hover:border-neutral-600 transition-colors whitespace-nowrap"
        >
          <span className="flex items-center gap-2 truncate">
            <Command className="w-3.5 h-3.5 text-[#2383E2] shrink-0" />
            <span className="truncate">{t.sidebar.testTriggerBtn}</span>
          </span>
          <span className="text-[11px] font-mono tabular-nums text-neutral-400 dark:text-neutral-500 shrink-0 ml-2">
            2× {modifier === 'Command' ? '⌘' : modifier === 'Control' ? 'Ctrl' : 'Alt'}
          </span>
        </button>

        {/* System Tray Preview Toggle */}
        <div className="relative">
          <button
            onClick={() => setTrayOpen((prev) => !prev)}
            className={`w-full flex items-center justify-between px-2.5 py-1.5 rounded-md text-[13px] transition-colors whitespace-nowrap ${
              trayOpen
                ? 'bg-[#EAEAE6] dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100'
                : 'text-neutral-600 dark:text-neutral-400 hover:bg-[#EFEFEC] dark:hover:bg-neutral-800/60'
            }`}
          >
            <span className="truncate">{t.sidebar.trayPreviewBtn}</span>
            <span className="text-[11px] font-mono tabular-nums text-neutral-400">
              {activeSlotCount}/8
            </span>
          </button>

          {trayOpen && (
            <div className="absolute bottom-full left-0 right-0 mb-2 z-40 rounded-lg bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-700 shadow-lg py-1.5 text-[12.5px]">
              <div className="px-3 py-1 text-[11px] text-neutral-400 dark:text-neutral-500 border-b border-neutral-100 dark:border-neutral-800 flex items-center justify-between">
                <span>{t.sidebar.trayTitle}</span>
                <span className="font-mono tabular-nums">{intervalMs}ms</span>
              </div>
              <button
                onClick={() => {
                  onNavigate('composer');
                  setTrayOpen(false);
                }}
                className="w-full px-3 py-1.5 text-left text-neutral-800 dark:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800 font-medium"
              >
                {t.sidebar.trayManage}
              </button>
              <button
                onClick={onToggleListenerPaused}
                className="w-full px-3 py-1.5 text-left text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 flex items-center justify-between"
              >
                <span>{listenerPaused ? t.sidebar.trayResume : t.sidebar.trayPause}</span>
                {listenerPaused ? <Play className="w-3 h-3 text-neutral-400" /> : <Pause className="w-3 h-3 text-neutral-400" />}
              </button>
              <button
                onClick={onToggleAutostart}
                className="w-full px-3 py-1.5 text-left text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 flex items-center justify-between"
              >
                <span>{t.sidebar.trayAutostart}</span>
                {autostart && <Check className="w-3.5 h-3.5 text-[#2383E2]" />}
              </button>
              <div className="my-1 border-t border-neutral-100 dark:border-neutral-800" />
              <button
                onClick={() => setTrayOpen(false)}
                className="w-full px-3 py-1.5 text-left text-neutral-500 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800 flex items-center justify-between"
              >
                <span>{t.sidebar.trayQuit}</span>
                <Power className="w-3 h-3" />
              </button>
            </div>
          )}
        </div>

        {/* Clean Metadata & Theme Row */}
        <div className="flex items-center justify-between px-2.5 pt-2 text-[12px] text-neutral-500 dark:text-neutral-400">
          <span className="tabular-nums">
            {t.sidebar.slotsCount} · {activeSlotCount}/8
          </span>
          <button
            onClick={onToggleDarkMode}
            className="flex items-center gap-1.5 px-2 py-1 rounded hover:bg-[#EAEAE6] dark:hover:bg-neutral-800 text-neutral-600 dark:text-neutral-300 transition-colors"
            title={t.sidebar.themeToggle}
          >
            {darkMode ? <Sun className="w-3.5 h-3.5" /> : <Moon className="w-3.5 h-3.5" />}
            <span>{darkMode ? t.sidebar.lightMode : t.sidebar.darkMode}</span>
          </button>
        </div>
      </div>
    </aside>
  );

  return (
    <>
      {/* Mobile floating menu trigger (only visible on small screens since there's no top bar) */}
      <button
        onClick={() => onSetMobileOpen(true)}
        className="md:hidden fixed top-4 left-4 z-30 p-2 rounded-lg bg-white/90 dark:bg-neutral-900/90 border border-neutral-200 dark:border-neutral-700 shadow-sm text-neutral-700 dark:text-neutral-200"
        aria-label="Open menu"
      >
        <Menu className="w-4 h-4" />
      </button>

      {/* Desktop persistent sidebar */}
      <div className="hidden md:block h-screen sticky top-0 shrink-0">
        {sidebarContent}
      </div>

      {/* Mobile drawer */}
      {mobileOpen && (
        <div className="md:hidden fixed inset-0 z-40 flex">
          <div className="h-full">{sidebarContent}</div>
          <div
            className="flex-1 bg-black/25"
            onClick={() => onSetMobileOpen(false)}
          />
        </div>
      )}
    </>
  );
};

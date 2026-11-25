import React from 'react';
import { Folder, FileText, Terminal, Play, X } from 'lucide-react';
import {
  ContextEntry,
  ContextState,
  ExecutionRecord,
  KeyRingConfig,
} from '../types/keyring';
import { TranslationDict } from '../i18n/translations';
import { SANDBOX_ENTRIES } from '../data/presets';

interface SandboxPageProps {
  config: KeyRingConfig;
  t: TranslationDict;
  contextState: ContextState;
  onToggleEntry: (entry: ContextEntry, multiSelect: boolean) => void;
  onClearSelection: () => void;
  onTriggerAtCenter: () => void;
  executionLogs: ExecutionRecord[];
  activeTerminalSession: { cwd: string; command: string } | null;
  onCloseTerminal: () => void;
}

export const SandboxPage: React.FC<SandboxPageProps> = ({
  config,
  t,
  contextState,
  onToggleEntry,
  onClearSelection,
  onTriggerAtCenter,
  executionLogs,
  activeTerminalSession,
  onCloseTerminal,
}) => {
  const isSelected = (path: string) => contextState.selectedPaths.includes(path);
  const modifierBadge =
    config.trigger.modifier === 'Command'
      ? '⌘ Command'
      : config.trigger.modifier === 'Control'
      ? 'Ctrl'
      : 'Alt';

  return (
    <div className="max-w-4xl mx-auto px-6 md:px-14 py-10 md:py-12">
      <header className="pb-8 border-b border-neutral-200/80 dark:border-neutral-800">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div className="max-w-2xl">
            <h1 className="text-[28px] font-semibold tracking-tight text-neutral-900 dark:text-neutral-100 leading-tight">
              {t.sandbox.title}
            </h1>
            <p className="mt-2 text-[14px] text-neutral-500 dark:text-neutral-400 leading-relaxed">
              {t.sandbox.subtitle}
            </p>
          </div>

          <button
            onClick={onTriggerAtCenter}
            className="inline-flex items-center gap-2 px-4 py-2 rounded-lg text-[13px] font-medium bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900 hover:opacity-90 transition-opacity whitespace-nowrap shrink-0"
          >
            <Play className="w-3.5 h-3.5 fill-current" />
            <span>{t.sandbox.triggerNow}</span>
          </button>
        </div>
      </header>

      <div className="mt-8 grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">
        {/* Left: Simulated File Manager */}
        <section className="lg:col-span-7 rounded-xl border border-neutral-200/80 dark:border-neutral-800 bg-white dark:bg-[#1C1C1C] overflow-hidden">
          <div className="px-4 py-3 bg-[#FAFAF9] dark:bg-[#171717] border-b border-neutral-200/80 dark:border-neutral-800 flex flex-wrap items-center justify-between gap-2">
            <div>
              <div className="text-[13px] font-semibold text-neutral-800 dark:text-neutral-200">
                {t.sandbox.explorerHeader}
              </div>
              <div className="text-[11.5px] font-mono text-neutral-500 dark:text-neutral-400 mt-0.5">
                {contextState.currentDirectory}
              </div>
            </div>
            {contextState.selectedPaths.length > 0 && (
              <button
                onClick={onClearSelection}
                className="text-[12px] text-[#2383E2] hover:underline whitespace-nowrap"
              >
                {t.sandbox.clearSelection}
              </button>
            )}
          </div>

          <div className="divide-y divide-neutral-100 dark:divide-neutral-800/70">
            {SANDBOX_ENTRIES.map((entry) => {
              const selected = isSelected(entry.path);
              return (
                <div
                  key={entry.id}
                  onClick={(e) => onToggleEntry(entry, e.metaKey || e.ctrlKey || e.shiftKey)}
                  className={`px-4 py-2.5 flex items-center justify-between gap-3 cursor-pointer select-none transition-colors ${
                    selected
                      ? 'bg-[#2383E2]/10 text-neutral-900 dark:text-neutral-100'
                      : 'hover:bg-[#F7F7F5] dark:hover:bg-neutral-800/50 text-neutral-700 dark:text-neutral-300'
                  }`}
                >
                  <div className="flex items-center gap-3 min-w-0">
                    <input
                      type="checkbox"
                      checked={selected}
                      onChange={() => {}}
                      onClick={(e) => {
                        e.stopPropagation();
                        onToggleEntry(entry, true);
                      }}
                      className="rounded border-neutral-300 dark:border-neutral-600 accent-[#2383E2]"
                      aria-label={entry.name}
                    />
                    {entry.kind === 'folder' ? (
                      <Folder className="w-4 h-4 text-[#2383E2] shrink-0" />
                    ) : (
                      <FileText className="w-4 h-4 text-neutral-400 shrink-0" />
                    )}
                    <span className="text-[13.5px] font-medium truncate">
                      {entry.name}
                    </span>
                  </div>

                  <div className="flex items-center gap-4 text-[12px] text-neutral-400 font-mono tabular-nums shrink-0">
                    <span>{entry.size}</span>
                    <span className="hidden sm:inline">{entry.modified}</span>
                  </div>
                </div>
              );
            })}
          </div>

          <div className="px-4 py-2.5 bg-[#FAFAF9] dark:bg-[#171717] border-t border-neutral-200/80 dark:border-neutral-800 flex items-center justify-between text-[12px] text-neutral-500 dark:text-neutral-400">
            <span>
              {t.sandbox.selectedCountLabel}:{' '}
              <strong className="font-mono tabular-nums text-neutral-800 dark:text-neutral-200">
                {contextState.selectedPaths.length}
              </strong>
            </span>
            <span className="font-mono text-[11.5px]">
              2× {modifierBadge} ({config.trigger.intervalMs}ms)
            </span>
          </div>
        </section>

        {/* Right: ContextState Snapshot & Execution History */}
        <section className="lg:col-span-5 space-y-6">
          {/* ContextState Snapshot */}
          <div>
            <h2 className="text-[13.5px] font-semibold text-neutral-900 dark:text-neutral-100 mb-2">
              {t.sandbox.contextPayloadTitle}
            </h2>
            <pre className="rounded-xl border border-neutral-200/80 dark:border-neutral-800 bg-[#FAFAF9] dark:bg-[#171717] p-3.5 text-[11.5px] font-mono text-neutral-700 dark:text-neutral-300 overflow-x-auto leading-relaxed">
              {JSON.stringify(
                {
                  selected_paths: contextState.selectedPaths,
                  current_directory: contextState.currentDirectory,
                },
                null,
                2
              )}
            </pre>
          </div>

          {/* Simulated Terminal Session */}
          {activeTerminalSession && (
            <div className="rounded-xl border border-neutral-800 bg-[#141414] text-neutral-100 overflow-hidden shadow-md">
              <div className="px-3.5 py-2 bg-[#1E1E1E] border-b border-neutral-800 flex items-center justify-between">
                <div className="flex items-center gap-2 text-[12px] font-medium text-neutral-300">
                  <Terminal className="w-3.5 h-3.5 text-[#4ADE80]" />
                  <span>{t.sandbox.terminalWindowTitle}</span>
                </div>
                <button
                  onClick={onCloseTerminal}
                  className="p-1 rounded hover:bg-neutral-700 text-neutral-400 hover:text-white"
                  title={t.sandbox.closeTerminal}
                >
                  <X className="w-3.5 h-3.5" />
                </button>
              </div>
              <div className="p-3.5 font-mono text-[12px] space-y-1.5 leading-relaxed">
                <div className="text-neutral-400">$ {activeTerminalSession.command}</div>
                <div className="text-[#4ADE80]">
                  {activeTerminalSession.cwd} % <span className="animate-pulse">_</span>
                </div>
              </div>
            </div>
          )}

          {/* Execution History */}
          <div>
            <h2 className="text-[13.5px] font-semibold text-neutral-900 dark:text-neutral-100 mb-2">
              {t.sandbox.executionLogTitle}
            </h2>
            {executionLogs.length === 0 ? (
              <div className="rounded-xl border border-neutral-200/80 dark:border-neutral-800 bg-[#FAFAF9] dark:bg-[#171717] p-4 text-[12.5px] text-neutral-500 dark:text-neutral-400 leading-relaxed">
                {t.sandbox.noExecutionsYet}
              </div>
            ) : (
              <div className="divide-y divide-neutral-200/70 dark:divide-neutral-800 border border-neutral-200/80 dark:border-neutral-800 rounded-xl bg-white dark:bg-[#1C1C1C]">
                {executionLogs.map((log) => (
                  <div key={log.id} className="p-3 space-y-1">
                    <div className="flex items-center justify-between text-[12.5px]">
                      <span className="font-medium text-neutral-900 dark:text-neutral-100">
                        {log.label}
                      </span>
                      <span className="font-mono tabular-nums text-[11px] text-neutral-400">
                        {log.timestamp}
                      </span>
                    </div>
                    <div className="text-[11.5px] font-mono text-neutral-500 dark:text-neutral-400 break-all">
                      {log.commandPreview}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </section>
      </div>
    </div>
  );
};

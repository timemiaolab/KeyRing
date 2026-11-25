import React, { useState } from 'react';
import { Check, FolderPlus, LockKeyhole, Plus, Trash2 } from 'lucide-react';
import { ActiveSlot, KeyRingConfig, ModifierKey } from '../types/keyring';
import { COMMON_IDE_PRESETS } from '../data/presets';
import { AppBrandIcon, getSlotIconProps } from './AppBrandIcon';
import { PieSvgRing, getSlotDisplayName } from './PieSvgRing';
import { TranslationDict } from '../i18n/translations';
import { RuntimeStatus, openAccessibilitySettings, requestAccessibilityPermission } from '../tauriBridge';

interface ConfigPageProps {
  config: KeyRingConfig;
  t: TranslationDict;
  darkMode: boolean;
  status: RuntimeStatus | null;
  onStatusChange: (status: RuntimeStatus | null) => void;
  onUpdateConfig: (next: KeyRingConfig | ((prev: KeyRingConfig) => KeyRingConfig)) => void;
  onToggleDarkMode: () => void;
}

const MODIFIERS: { id: ModifierKey; label: string }[] = [
  { id: 'Command', label: 'Command（⌘）' },
  { id: 'Control', label: 'Control（Ctrl）' },
  { id: 'Alt', label: 'Option / Alt' },
];

export const ConfigPage: React.FC<ConfigPageProps> = ({
  config,
  t,
  darkMode,
  status,
  onStatusChange,
  onUpdateConfig,
  onToggleDarkMode,
}) => {
  const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);
  const [adding, setAdding] = useState(false);
  const needsPermission = status?.platform === 'macos' && status.accessibilityTrusted === false;
  const listenerOk = Boolean(status?.listenerRunning) && !status?.paused;

  const removeSlot = (id: string) => {
    onUpdateConfig((prev) => ({
      ...prev,
      activeSlots: prev.activeSlots.filter((slot) => slot.id !== id),
    }));
  };

  const addPreset = (presetId: string) => {
    const preset = COMMON_IDE_PRESETS.find((item) => item.id === presetId);
    if (!preset) return;
    onUpdateConfig((prev) => {
      if (prev.activeSlots.some((slot) => slot.id === preset.id)) {
        return prev;
      }
      const slot: ActiveSlot = {
        id: preset.id,
        type: 'app',
        name: preset.name,
        execPath: preset.execPath,
        iconCache: preset.iconCache,
        presetKey: preset.presetKey,
      };
      return { ...prev, activeSlots: [...prev.activeSlots, slot] };
    });
    setAdding(false);
  };

  const addBuiltin = (action: 'open_in_terminal' | 'copy_path') => {
    const id = action === 'open_in_terminal' ? 'builtin-terminal' : 'builtin-copy';
    onUpdateConfig((prev) => {
      if (prev.activeSlots.some((slot) => slot.id === id)) {
        return prev;
      }
      return {
        ...prev,
        activeSlots: [...prev.activeSlots, { id, type: 'builtin', action }],
      };
    });
    setAdding(false);
  };

  const handleRequestPermission = async () => {
    const next = await requestAccessibilityPermission();
    onStatusChange(next);
    if (next && !next.accessibilityTrusted) {
      await openAccessibilitySettings();
    }
  };

  return (
    <div className="flex-1 min-h-0 overflow-y-auto">
      <div className="max-w-4xl mx-auto px-8 py-6 space-y-6">
        {needsPermission && (
          <div className="rounded-2xl border border-amber-200 bg-amber-50 dark:bg-amber-950/30 dark:border-amber-900/70 px-5 py-4">
            <div className="flex items-start gap-3">
              <LockKeyhole className="w-5 h-5 text-amber-600 dark:text-amber-400 shrink-0 mt-0.5" />
              <div className="min-w-0">
                <div className="text-[15px] font-medium text-amber-950 dark:text-amber-100">
                  需要系统授权，才能双击 Command 弹出圆环
                </div>
                <p className="mt-1.5 text-[13px] leading-relaxed text-amber-900/80 dark:text-amber-200/80">
                  KeyRing 会在后台监听双击 Command，但不会改你的键盘设置。请打开「系统设置」→「隐私与安全性」→「辅助功能」，打开 KeyRing 开关。授权后回到这里，监听会自动重新开始。
                </p>
                <div className="mt-3 flex flex-wrap gap-2">
                  <button
                    type="button"
                    onClick={() => void handleRequestPermission()}
                    className="px-3 py-1.5 rounded-lg text-[13px] font-medium bg-amber-900 text-white hover:bg-amber-800"
                  >
                    打开系统设置并请求授权
                  </button>
                </div>
                {status?.listenerError && (
                  <p className="mt-2 text-[12px] text-amber-800 dark:text-amber-300">{status.listenerError}</p>
                )}
              </div>
            </div>
          </div>
        )}

        <section className="grid md:grid-cols-[1.1fr_0.9fr] gap-8 items-start">
          <div>
            <h2 className="text-[20px] font-semibold tracking-tight">圆环上的动作</h2>
            <p className="mt-1 text-[13px] text-neutral-500 dark:text-neutral-400 leading-relaxed">
              在访达里选中文件或文件夹，双击 Command，再点圆环上的选项。例如点 Cursor
              会用 Cursor 打开选中项；点「在终端打开」会打开终端并进入该文件夹（若选的是文件，则进入它所在的文件夹）。
            </p>

            <ul className="mt-4 divide-y divide-neutral-200/80 dark:divide-neutral-800 rounded-2xl border border-neutral-200/80 dark:border-neutral-800 overflow-hidden">
              {config.activeSlots.length === 0 && (
                <li className="px-4 py-6 text-[13px] text-neutral-500">还没有动作，先添加一个吧。</li>
              )}
              {config.activeSlots.map((slot, index) => (
                <li key={slot.id} className="flex items-center gap-3 px-3 py-2.5 bg-white dark:bg-[#1F1F1F]">
                  <span className="w-5 text-center text-[11px] text-neutral-400">{index + 1}</span>
                  <AppBrandIcon {...getSlotIconProps(slot)} size={28} />
                  <span className="flex-1 min-w-0 text-[14px] truncate">{getSlotDisplayName(slot, t)}</span>
                  <button
                    type="button"
                    aria-label="移出圆环"
                    onClick={() => removeSlot(slot.id)}
                    className="w-8 h-8 rounded-md flex items-center justify-center text-neutral-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-950/40"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                  </button>
                </li>
              ))}
            </ul>

            <div className="mt-3">
              {!adding ? (
                <button
                  type="button"
                  onClick={() => setAdding(true)}
                  className="inline-flex items-center gap-1.5 text-[13px] font-medium text-neutral-700 dark:text-neutral-200 hover:text-neutral-950"
                >
                  <Plus className="w-4 h-4" />
                  添加动作
                </button>
              ) : (
                <div className="rounded-xl border border-neutral-200 dark:border-neutral-800 p-3 space-y-2">
                  <div className="text-[12px] text-neutral-500">常用应用</div>
                  <div className="flex flex-wrap gap-2">
                    {COMMON_IDE_PRESETS.map((preset) => {
                      const exists = config.activeSlots.some((slot) => slot.id === preset.id);
                      return (
                        <button
                          key={preset.id}
                          type="button"
                          disabled={exists}
                          onClick={() => addPreset(preset.id)}
                          className="px-2.5 py-1 rounded-lg text-[12px] border border-neutral-200 dark:border-neutral-700 disabled:opacity-40 hover:bg-neutral-50 dark:hover:bg-neutral-800"
                        >
                          {preset.name}
                        </button>
                      );
                    })}
                  </div>
                  <div className="flex flex-wrap gap-2 pt-1">
                    <button
                      type="button"
                      onClick={() => addBuiltin('open_in_terminal')}
                      className="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg text-[12px] border border-neutral-200 dark:border-neutral-700 hover:bg-neutral-50 dark:hover:bg-neutral-800"
                    >
                      <FolderPlus className="w-3.5 h-3.5" />
                      在终端打开
                    </button>
                    <button
                      type="button"
                      onClick={() => addBuiltin('copy_path')}
                      className="px-2.5 py-1 rounded-lg text-[12px] border border-neutral-200 dark:border-neutral-700 hover:bg-neutral-50 dark:hover:bg-neutral-800"
                    >
                      复制路径
                    </button>
                    <button
                      type="button"
                      onClick={() => setAdding(false)}
                      className="px-2.5 py-1 text-[12px] text-neutral-500"
                    >
                      完成
                    </button>
                  </div>
                </div>
              )}
            </div>
          </div>

          <div className="flex flex-col items-center justify-center min-h-[280px]">
            <PieSvgRing
              slots={config.activeSlots}
              ringDiameter={config.ui.ringDiameter}
              innerHoleDiameter={config.ui.innerHoleDiameter}
              iconSize={config.ui.iconSize}
              t={t}
              hoveredIndex={hoveredIndex}
              onHoverIndex={setHoveredIndex}
              onSelectSlot={() => undefined}
              onCenterClick={() => undefined}
              darkSurface={darkMode}
            />
            <p className="mt-3 text-[12px] text-neutral-400">预览，不会真正执行</p>
          </div>
        </section>

        <section className="rounded-2xl border border-neutral-200/80 dark:border-neutral-800 p-5 space-y-4">
          <h2 className="text-[16px] font-semibold">触发与状态</h2>
          <div className="grid sm:grid-cols-2 gap-4">
            <label className="block text-[13px]">
              <span className="text-neutral-500">双击的修饰键</span>
              <select
                className="mt-1 w-full h-9 rounded-lg border border-neutral-200 dark:border-neutral-700 bg-transparent px-2"
                value={config.trigger.modifier}
                onChange={(e) =>
                  onUpdateConfig((prev) => ({
                    ...prev,
                    trigger: { ...prev.trigger, modifier: e.target.value as ModifierKey },
                  }))
                }
              >
                {MODIFIERS.map((item) => (
                  <option key={item.id} value={item.id}>
                    {item.label}
                  </option>
                ))}
              </select>
            </label>
            <label className="block text-[13px]">
              <span className="text-neutral-500">两次按下的间隔（毫秒）</span>
              <input
                type="number"
                min={160}
                max={800}
                step={20}
                className="mt-1 w-full h-9 rounded-lg border border-neutral-200 dark:border-neutral-700 bg-transparent px-2"
                value={config.trigger.intervalMs}
                onChange={(e) =>
                  onUpdateConfig((prev) => ({
                    ...prev,
                    trigger: { ...prev.trigger, intervalMs: Number(e.target.value) || 300 },
                  }))
                }
              />
            </label>
          </div>

          <div className="flex items-center gap-2 text-[13px]">
            <span
              className={`inline-flex items-center gap-1.5 px-2 py-1 rounded-full ${
                listenerOk
                  ? 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950/50 dark:text-emerald-300'
                  : 'bg-neutral-100 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300'
              }`}
            >
              <Check className="w-3.5 h-3.5" />
              {status?.paused
                ? '监听已暂停'
                : listenerOk
                  ? '正在监听双击'
                  : needsPermission
                    ? '等待系统授权'
                    : '监听尚未就绪'}
            </span>
            {status?.platform === 'macos' && status.accessibilityTrusted && (
              <span className="text-neutral-400">辅助功能已授权</span>
            )}
          </div>

          <div className="flex flex-wrap items-center gap-5 pt-1 text-[13px]">
            <label className="inline-flex items-center gap-2">
              <input
                type="checkbox"
                checked={config.autostart}
                onChange={() => onUpdateConfig((prev) => ({ ...prev, autostart: !prev.autostart }))}
              />
              开机时自动启动
            </label>
            <label className="inline-flex items-center gap-2">
              <input type="checkbox" checked={darkMode} onChange={onToggleDarkMode} />
              深色外观
            </label>
          </div>
        </section>
      </div>
    </div>
  );
};

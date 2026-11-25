import React, { useState } from 'react';
import { ArrowDown, ArrowUp, GripVertical, Play, Trash2, ArrowRight } from 'lucide-react';
import { ActiveSlot, KeyRingConfig } from '../types/keyring';
import { TranslationDict } from '../i18n/translations';
import { PieSvgRing, getSlotDisplayName } from './PieSvgRing';
import { AppBrandIcon, getSlotIconProps } from './AppBrandIcon';

interface RingComposerPageProps {
  config: KeyRingConfig;
  t: TranslationDict;
  darkMode: boolean;
  onReorderSlots: (nextSlots: ActiveSlot[]) => void;
  onRemoveSlot: (slotId: string) => void;
  onExecuteSlot: (slot: ActiveSlot) => void;
  onPopoutTest: () => void;
  onNavigateLibrary: () => void;
}

export const RingComposerPage: React.FC<RingComposerPageProps> = ({
  config,
  t,
  darkMode,
  onReorderSlots,
  onRemoveSlot,
  onExecuteSlot,
  onPopoutTest,
  onNavigateLibrary,
}) => {
  const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);
  const [dragIndex, setDragIndex] = useState<number | null>(null);
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null);

  const slots = config.activeSlots;
  const count = slots.length;

  const getDirectionLabel = (index: number, total: number): string => {
    if (total === 8) {
      return t.directions[index] || `${index + 1}`;
    }
    const deg = Math.round((index * 360) / Math.max(total, 1));
    return `${deg}°`;
  };

  const moveSlot = (from: number, to: number) => {
    if (to < 0 || to >= slots.length || from === to) return;
    const next = [...slots];
    const [moved] = next.splice(from, 1);
    next.splice(to, 0, moved);
    onReorderSlots(next);
  };

  const handleDrop = (targetIndex: number) => {
    if (dragIndex !== null && dragIndex !== targetIndex) {
      moveSlot(dragIndex, targetIndex);
    }
    setDragIndex(null);
    setDragOverIndex(null);
  };

  return (
    <div className="max-w-4xl mx-auto px-6 md:px-14 py-10 md:py-12">
      {/* Notion-style Document Title Header (No top bar) */}
      <header className="pb-8 border-b border-neutral-200/80 dark:border-neutral-800">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div className="max-w-2xl">
            <h1 className="text-[28px] font-semibold tracking-tight text-neutral-900 dark:text-neutral-100 leading-tight">
              {t.composer.title}
            </h1>
            <p className="mt-2 text-[14px] text-neutral-500 dark:text-neutral-400 leading-relaxed">
              {t.composer.subtitle}
            </p>
          </div>

          <button
            onClick={onPopoutTest}
            className="inline-flex items-center gap-2 px-4 py-2 rounded-lg text-[13px] font-medium bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900 hover:bg-neutral-800 dark:hover:bg-neutral-200 transition-colors whitespace-nowrap shrink-0"
          >
            <Play className="w-3.5 h-3.5 fill-current" />
            <span>{t.composer.popoutTest}</span>
          </button>
        </div>
      </header>

      {/* Two-Column Interactive Composer */}
      <div className="mt-10 grid grid-cols-1 lg:grid-cols-12 gap-10 items-start">
        {/* Left: Live 8-Slot Pie Preview */}
        <section className="lg:col-span-6 flex flex-col items-center rounded-xl border border-neutral-200/80 dark:border-neutral-800 bg-[#FAFAF9] dark:bg-[#171717] p-6">
          <div className="w-full flex items-center justify-between mb-4">
            <div>
              <h2 className="text-[14px] font-semibold text-neutral-900 dark:text-neutral-100">
                {t.composer.previewTitle}
              </h2>
              <p className="text-[12px] text-neutral-500 dark:text-neutral-400 mt-0.5">
                {t.composer.previewHint}
              </p>
            </div>
            <span className="text-[12px] font-mono tabular-nums text-neutral-400 dark:text-neutral-500 shrink-0">
              {count} / 8 · {count > 0 ? `${Math.round(360 / count)}°` : '—'}
            </span>
          </div>

          <div className="py-4 flex items-center justify-center min-h-[310px]">
            <PieSvgRing
              slots={slots}
              ringDiameter={config.ui.ringDiameter}
              innerHoleDiameter={config.ui.innerHoleDiameter}
              iconSize={config.ui.iconSize}
              t={t}
              hoveredIndex={hoveredIndex}
              onHoverIndex={setHoveredIndex}
              onSelectSlot={(slot) => onExecuteSlot(slot)}
              darkSurface={darkMode}
            />
          </div>
        </section>

        {/* Right: Draggable Active Slot List */}
        <section className="lg:col-span-6">
          <div className="flex items-center justify-between mb-3">
            <div>
              <h2 className="text-[14px] font-semibold text-neutral-900 dark:text-neutral-100">
                {t.composer.activeListTitle}
              </h2>
              <p className="text-[12px] text-neutral-500 dark:text-neutral-400 mt-0.5">
                {t.composer.activeListDesc}
              </p>
            </div>
            {count >= 8 && (
              <span className="text-[12px] text-amber-600 dark:text-amber-400 font-medium whitespace-nowrap">
                {t.composer.ringFullHint}
              </span>
            )}
          </div>

          {count === 0 ? (
            <div className="rounded-xl border border-dashed border-neutral-300 dark:border-neutral-700 p-8 text-center">
              <p className="text-[14px] font-medium text-neutral-800 dark:text-neutral-200">
                {t.composer.emptyTitle}
              </p>
              <p className="text-[13px] text-neutral-500 dark:text-neutral-400 mt-1 mb-4">
                {t.composer.emptyDesc}
              </p>
              <button
                onClick={onNavigateLibrary}
                className="inline-flex items-center gap-1.5 px-4 py-2 rounded-lg text-[13px] font-medium bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900 hover:opacity-90 transition-opacity"
              >
                <span>{t.composer.goToLibrary}</span>
                <ArrowRight className="w-3.5 h-3.5" />
              </button>
            </div>
          ) : (
            <div className="divide-y divide-neutral-200/70 dark:divide-neutral-800 border border-neutral-200/80 dark:border-neutral-800 rounded-xl bg-white dark:bg-[#1C1C1C]">
              {slots.map((slot, index) => {
                const isHovered = hoveredIndex === index;
                const isDragOver = dragOverIndex === index && dragIndex !== index;
                const label = getSlotDisplayName(slot, t);
                const dirLabel = getDirectionLabel(index, count);

                return (
                  <div
                    key={slot.id}
                    draggable
                    onDragStart={() => setDragIndex(index)}
                    onDragOver={(e) => {
                      e.preventDefault();
                      if (dragOverIndex !== index) setDragOverIndex(index);
                    }}
                    onDragLeave={() => {
                      if (dragOverIndex === index) setDragOverIndex(null);
                    }}
                    onDrop={() => handleDrop(index)}
                    onDragEnd={() => {
                      setDragIndex(null);
                      setDragOverIndex(null);
                    }}
                    onMouseEnter={() => setHoveredIndex(index)}
                    onMouseLeave={() => setHoveredIndex(null)}
                    className={`flex items-center justify-between gap-3 px-3.5 py-2.5 transition-colors first:rounded-t-xl last:rounded-b-xl ${
                      isDragOver
                        ? 'bg-[#2383E2]/10'
                        : isHovered
                        ? 'bg-[#F7F7F5] dark:bg-neutral-800/70'
                        : ''
                    }`}
                  >
                    <div className="flex items-center gap-3 min-w-0">
                      <span
                        className="cursor-grab active:cursor-grabbing text-neutral-300 dark:text-neutral-600 hover:text-neutral-500 dark:hover:text-neutral-400 shrink-0"
                        title={t.composer.dragHint}
                      >
                        <GripVertical className="w-4 h-4" />
                      </span>

                      <AppBrandIcon {...getSlotIconProps(slot)} size={28} />

                      <div className="min-w-0">
                        <div className="text-[13.5px] font-medium text-neutral-900 dark:text-neutral-100 truncate">
                          {label}
                        </div>
                        <div className="text-[11.5px] text-neutral-500 dark:text-neutral-400 truncate font-mono tabular-nums">
                          #{index + 1} · {t.composer.slotDirectionLabel} {dirLabel} ·{' '}
                          {slot.type === 'builtin' ? 'builtin' : slot.execPath}
                        </div>
                      </div>
                    </div>

                    <div className="flex items-center gap-1 shrink-0">
                      <button
                        onClick={() => moveSlot(index, index - 1)}
                        disabled={index === 0}
                        title={t.composer.moveUp}
                        className="p-1.5 rounded text-neutral-400 hover:text-neutral-800 dark:hover:text-neutral-200 hover:bg-neutral-200/60 dark:hover:bg-neutral-700 disabled:opacity-25 disabled:pointer-events-none transition-colors"
                      >
                        <ArrowUp className="w-3.5 h-3.5" />
                      </button>
                      <button
                        onClick={() => moveSlot(index, index + 1)}
                        disabled={index === count - 1}
                        title={t.composer.moveDown}
                        className="p-1.5 rounded text-neutral-400 hover:text-neutral-800 dark:hover:text-neutral-200 hover:bg-neutral-200/60 dark:hover:bg-neutral-700 disabled:opacity-25 disabled:pointer-events-none transition-colors"
                      >
                        <ArrowDown className="w-3.5 h-3.5" />
                      </button>
                      <button
                        onClick={() => onRemoveSlot(slot.id)}
                        title={t.composer.removeFromRing}
                        className="p-1.5 rounded text-neutral-400 hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/40 transition-colors"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </section>
      </div>
    </div>
  );
};

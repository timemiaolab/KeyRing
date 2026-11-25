import React, { useEffect, useState } from 'react';
import { ActiveSlot } from '../types/keyring';
import { TranslationDict } from '../i18n/translations';
import { PieSvgRing } from './PieSvgRing';

interface PieMenuOverlayProps {
  visible: boolean;
  coords: { x: number; y: number };
  slots: ActiveSlot[];
  ringDiameter: number;
  innerHoleDiameter: number;
  iconSize: number;
  t: TranslationDict;
  darkSurface: boolean;
  onExecute: (slot: ActiveSlot) => void;
  onClose: () => void;
}

export const PieMenuOverlay: React.FC<PieMenuOverlayProps> = ({
  visible,
  coords,
  slots,
  ringDiameter,
  innerHoleDiameter,
  iconSize,
  t,
  darkSurface,
  onExecute,
  onClose,
}) => {
  const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);

  useEffect(() => {
    if (!visible) {
      setHoveredIndex(null);
      return;
    }

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        onClose();
      }
    };

    const handleBlur = () => {
      onClose();
    };

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('blur', handleBlur);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('blur', handleBlur);
    };
  }, [visible, onClose]);

  if (!visible) return null;

  // Screen boundary clamping with safety margin
  const pad = 22;
  const totalHalf = ringDiameter / 2 + pad;
  const safetyMargin = 16;
  const vw = typeof window !== 'undefined' ? window.innerWidth : 1280;
  const vh = typeof window !== 'undefined' ? window.innerHeight : 800;

  const clampedX = Math.max(
    totalHalf + safetyMargin,
    Math.min(vw - totalHalf - safetyMargin, coords.x)
  );
  const clampedY = Math.max(
    totalHalf + safetyMargin,
    Math.min(vh - totalHalf - safetyMargin, coords.y)
  );

  return (
    <div
      className="fixed inset-0 z-50 bg-neutral-950/10 dark:bg-black/30 backdrop-blur-[1px]"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) {
          onClose();
        }
      }}
    >
      <div
        style={{
          left: clampedX,
          top: clampedY,
          transform: 'translate(-50%, -50%)',
        }}
        className="fixed animate-pie-in"
      >
        <PieSvgRing
          slots={slots}
          ringDiameter={ringDiameter}
          innerHoleDiameter={innerHoleDiameter}
          iconSize={iconSize}
          t={t}
          hoveredIndex={hoveredIndex}
          onHoverIndex={setHoveredIndex}
          onSelectSlot={(slot) => {
            onExecute(slot);
          }}
          onCenterClick={onClose}
          darkSurface={darkSurface}
        />

        <div className="mt-1 text-center pointer-events-none">
          <span className="inline-block rounded-md bg-white/90 dark:bg-neutral-900/90 border border-neutral-200/80 dark:border-neutral-800 px-2 py-0.5 text-[11px] text-neutral-500 dark:text-neutral-400 shadow-xs">
            {slots.length === 0 ? t.pie.noActiveSlots : t.pie.escHint}
          </span>
        </div>
      </div>
    </div>
  );
};

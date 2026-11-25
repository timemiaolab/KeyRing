import React from 'react';
import { ActiveSlot } from '../types/keyring';
import { TranslationDict } from '../i18n/translations';
import { AppBrandIcon, getSlotIconProps } from './AppBrandIcon';

interface PieSvgRingProps {
  slots: ActiveSlot[];
  ringDiameter: number;
  innerHoleDiameter: number;
  iconSize: number;
  t: TranslationDict;
  hoveredIndex: number | null;
  onHoverIndex: (index: number | null) => void;
  onSelectSlot?: (slot: ActiveSlot, index: number) => void;
  onCenterClick?: () => void;
  centerOverrideText?: string | null;
  darkSurface?: boolean;
}

function polarToCartesian(cx: number, cy: number, r: number, angleInDegrees: number) {
  const angleInRadians = (angleInDegrees * Math.PI) / 180.0;
  return {
    x: cx + r * Math.cos(angleInRadians),
    y: cy + r * Math.sin(angleInRadians),
  };
}

function createDonutSectorPath(
  cx: number,
  cy: number,
  rInner: number,
  rOuter: number,
  startAngle: number,
  endAngle: number
): string {
  const outerStart = polarToCartesian(cx, cy, rOuter, startAngle);
  const outerEnd = polarToCartesian(cx, cy, rOuter, endAngle);
  const innerEnd = polarToCartesian(cx, cy, rInner, endAngle);
  const innerStart = polarToCartesian(cx, cy, rInner, startAngle);
  const largeArcFlag = endAngle - startAngle <= 180 ? '0' : '1';

  return [
    'M', outerStart.x.toFixed(3), outerStart.y.toFixed(3),
    'A', rOuter.toFixed(3), rOuter.toFixed(3), 0, largeArcFlag, 1, outerEnd.x.toFixed(3), outerEnd.y.toFixed(3),
    'L', innerEnd.x.toFixed(3), innerEnd.y.toFixed(3),
    'A', rInner.toFixed(3), rInner.toFixed(3), 0, largeArcFlag, 0, innerStart.x.toFixed(3), innerStart.y.toFixed(3),
    'Z',
  ].join(' ');
}

export function getSlotDisplayName(slot: ActiveSlot, t: TranslationDict): string {
  if (slot.type === 'builtin') {
    return t.builtins[slot.action];
  }
  return slot.name;
}

export const PieSvgRing: React.FC<PieSvgRingProps> = ({
  slots,
  ringDiameter,
  innerHoleDiameter,
  iconSize,
  t,
  hoveredIndex,
  onHoverIndex,
  onSelectSlot,
  onCenterClick,
  centerOverrideText,
  darkSurface = false,
}) => {
  // Extra canvas padding so hovered sector scale(1.06) never clips
  const pad = 22;
  const svgSize = ringDiameter + pad * 2;
  const cx = svgSize / 2;
  const cy = svgSize / 2;
  const outerR = ringDiameter / 2;
  const innerR = Math.min(innerHoleDiameter / 2, outerR - 36);
  const midR = (outerR + innerR) / 2;

  const count = Math.min(slots.length, 8);
  const sliceDeg = count > 0 ? 360 / count : 360;
  // Subtle gap between sectors when N >= 2
  const gapDeg = count >= 2 ? 1.8 : 0;

  const hoveredSlot = hoveredIndex !== null && slots[hoveredIndex] ? slots[hoveredIndex] : null;
  const centerLabel =
    centerOverrideText ||
    (hoveredSlot ? getSlotDisplayName(hoveredSlot, t) : t.composer.centerIdleHint);

  return (
    <div
      className="relative select-none"
      style={{ width: svgSize, height: svgSize }}
    >
      <svg
        width={svgSize}
        height={svgSize}
        viewBox={`0 0 ${svgSize} ${svgSize}`}
        className="overflow-visible drop-shadow-[0_14px_34px_rgba(0,0,0,0.18)]"
      >
        {/* Outer subtle halo ring */}
        <circle
          cx={cx}
          cy={cy}
          r={outerR + 1}
          fill="none"
          stroke={darkSurface ? 'rgba(255,255,255,0.08)' : 'rgba(15,23,42,0.07)'}
          strokeWidth="1"
        />

        {count === 0 && (
          <circle
            cx={cx}
            cy={cy}
            r={(outerR + innerR) / 2}
            fill="none"
            stroke={darkSurface ? 'rgba(255,255,255,0.1)' : 'rgba(15,23,42,0.08)'}
            strokeWidth={outerR - innerR}
            strokeDasharray="6 6"
          />
        )}

        {count === 1 && slots[0] && (
          <g
            onMouseEnter={() => onHoverIndex(0)}
            onMouseLeave={() => onHoverIndex(null)}
            onClick={(e) => {
              e.stopPropagation();
              onSelectSlot?.(slots[0], 0);
            }}
            style={{
              transformOrigin: `${cx}px ${cy}px`,
              transform: hoveredIndex === 0 ? 'scale(1.04)' : 'scale(1)',
              transition: 'transform 140ms cubic-bezier(0.16, 1, 0.3, 1)',
            }}
            className="cursor-pointer"
          >
            <path
              d={`
                M ${cx} ${cy - outerR}
                A ${outerR} ${outerR} 0 1 1 ${cx - 0.01} ${cy - outerR}
                Z
                M ${cx} ${cy - innerR}
                A ${innerR} ${innerR} 0 1 0 ${cx - 0.01} ${cy - innerR}
                Z
              `}
              fillRule="evenodd"
              fill={
                hoveredIndex === 0
                  ? darkSurface
                    ? '#2A2A2D'
                    : '#F2F6FC'
                  : darkSurface
                  ? 'rgba(30,30,32,0.94)'
                  : 'rgba(255,255,255,0.95)'
              }
              stroke={
                hoveredIndex === 0
                  ? '#2383E2'
                  : darkSurface
                  ? 'rgba(255,255,255,0.12)'
                  : 'rgba(15,23,42,0.1)'
              }
              strokeWidth={hoveredIndex === 0 ? 1.75 : 1}
            />
          </g>
        )}

        {count >= 2 &&
          slots.slice(0, 8).map((slot, i) => {
            const centerDeg = -90 + i * sliceDeg;
            const startAngle = centerDeg - sliceDeg / 2 + gapDeg / 2;
            const endAngle = centerDeg + sliceDeg / 2 - gapDeg / 2;
            const isHovered = hoveredIndex === i;
            const pathD = createDonutSectorPath(cx, cy, innerR, outerR, startAngle, endAngle);

            return (
              <g
                key={slot.id}
                onMouseEnter={() => onHoverIndex(i)}
                onMouseLeave={() => onHoverIndex(null)}
                onClick={(e) => {
                  e.stopPropagation();
                  onSelectSlot?.(slot, i);
                }}
                style={{
                  transformOrigin: `${cx}px ${cy}px`,
                  transform: isHovered ? 'scale(1.06)' : 'scale(1)',
                  transition: 'transform 140ms cubic-bezier(0.16, 1, 0.3, 1)',
                }}
                className="cursor-pointer"
              >
                <path
                  d={pathD}
                  fill={
                    isHovered
                      ? darkSurface
                        ? '#2B2F36'
                        : '#EEF5FD'
                      : darkSurface
                      ? 'rgba(28, 28, 30, 0.94)'
                      : 'rgba(255, 255, 255, 0.96)'
                  }
                  stroke={
                    isHovered
                      ? '#2383E2'
                      : darkSurface
                      ? 'rgba(255,255,255,0.12)'
                      : 'rgba(15,23,42,0.1)'
                  }
                  strokeWidth={isHovered ? 1.75 : 1}
                />
              </g>
            );
          })}

        {/* Center hollow / dismiss button hit target */}
        <circle
          cx={cx}
          cy={cy}
          r={Math.max(innerR - 4, 16)}
          fill={darkSurface ? 'rgba(20, 20, 22, 0.92)' : 'rgba(250, 250, 249, 0.94)'}
          stroke={darkSurface ? 'rgba(255,255,255,0.1)' : 'rgba(15,23,42,0.08)'}
          strokeWidth="1"
          onClick={(e) => {
            e.stopPropagation();
            onCenterClick?.();
          }}
          className={onCenterClick ? 'cursor-pointer' : ''}
        />
      </svg>

      {/* Pure Icon Layer positioned accurately at each sector centroid */}
      {slots.slice(0, 8).map((slot, i) => {
        const centerDeg = -90 + i * sliceDeg;
        const pos = polarToCartesian(cx, cy, midR, centerDeg);
        const isHovered = hoveredIndex === i;
        const scale = isHovered ? 1.12 : 1;

        return (
          <div
            key={`icon-${slot.id}`}
            style={{
              left: pos.x,
              top: pos.y,
              width: iconSize,
              height: iconSize,
              transform: `translate(-50%, -50%) scale(${scale})`,
              transition: 'transform 140ms cubic-bezier(0.16, 1, 0.3, 1)',
            }}
            className="pointer-events-none absolute flex items-center justify-center"
          >
            <AppBrandIcon
              {...getSlotIconProps(slot)}
              size={iconSize}
            />
          </div>
        );
      })}

      {/* Center Hollow Label Overlay */}
      <div
        onClick={(e) => {
          e.stopPropagation();
          onCenterClick?.();
        }}
        style={{
          left: cx,
          top: cy,
          width: Math.max(innerR * 1.85, 56),
          height: Math.max(innerR * 1.85, 56),
          transform: 'translate(-50%, -50%)',
        }}
        className={`absolute flex items-center justify-center rounded-full px-2 text-center ${
          onCenterClick ? 'cursor-pointer' : 'pointer-events-none'
        }`}
      >
        <span
          className={`text-[11px] font-medium leading-tight line-clamp-2 transition-colors duration-100 ${
            hoveredSlot || centerOverrideText
              ? darkSurface
                ? 'text-neutral-100'
                : 'text-neutral-900'
              : darkSurface
              ? 'text-neutral-500'
              : 'text-neutral-400'
          }`}
        >
          {centerLabel}
        </span>
      </div>
    </div>
  );
};

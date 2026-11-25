import React from 'react';
import { ClipboardCopy, Terminal, AppWindow } from 'lucide-react';
import { ActiveSlot, BuiltinAction } from '../types/keyring';

interface AppBrandIconProps {
  presetKey?: string;
  builtinAction?: BuiltinAction;
  customIconDataUrl?: string;
  name?: string;
  size?: number;
  className?: string;
}

export const AppBrandIcon: React.FC<AppBrandIconProps> = ({
  presetKey,
  builtinAction,
  customIconDataUrl,
  name,
  size = 28,
  className = '',
}) => {
  if (customIconDataUrl) {
    return (
      <img
        src={customIconDataUrl}
        alt={name || 'App icon'}
        referrerPolicy="no-referrer"
        style={{ width: size, height: size }}
        className={`rounded-lg object-cover shrink-0 ${className}`}
      />
    );
  }

  if (builtinAction === 'copy_path') {
    return (
      <div
        style={{ width: size, height: size }}
        className={`rounded-lg bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900 flex items-center justify-center shrink-0 ${className}`}
      >
        <ClipboardCopy style={{ width: size * 0.56, height: size * 0.56 }} strokeWidth={2} />
      </div>
    );
  }

  if (builtinAction === 'open_in_terminal') {
    return (
      <div
        style={{ width: size, height: size }}
        className={`rounded-lg bg-[#1E1E1E] text-[#4ADE80] border border-neutral-700/60 flex items-center justify-center shrink-0 ${className}`}
      >
        <Terminal style={{ width: size * 0.56, height: size * 0.56 }} strokeWidth={2.2} />
      </div>
    );
  }

  switch (presetKey) {
    case 'cursor':
      return (
        <svg width={size} height={size} viewBox="0 0 36 36" fill="none" className={`shrink-0 ${className}`}>
          <rect width="36" height="36" rx="8" fill="#111111" />
          <path d="M18 6L28.5 12V24L18 30L7.5 24V12L18 6Z" stroke="#FFFFFF" strokeWidth="1.6" fill="#1C1C1C" />
          <path d="M18 6V18M18 18L28.5 12M18 18L7.5 24" stroke="#E5E5E5" strokeWidth="1.5" strokeLinecap="round" />
          <path d="M18 18L26 22.5L18 27L13 19.5L18 18Z" fill="#FFFFFF" />
        </svg>
      );
    case 'vscode':
      return (
        <svg width={size} height={size} viewBox="0 0 36 36" fill="none" className={`shrink-0 ${className}`}>
          <rect width="36" height="36" rx="8" fill="#1E293B" />
          <path d="M24.5 7.5L11.5 19.5L8 16.8L6.5 18L10 21L6.5 24L8 25.2L11.5 22.5L24.5 28.5L29 26.3V9.7L24.5 7.5Z" fill="#007ACC" />
          <path d="M24.5 7.5V28.5L29 26.3V9.7L24.5 7.5Z" fill="#38BDF8" />
        </svg>
      );
    case 'intellij':
      return (
        <svg width={size} height={size} viewBox="0 0 36 36" fill="none" className={`shrink-0 ${className}`}>
          <rect width="36" height="36" rx="8" fill="#18181B" />
          <rect x="6" y="6" width="24" height="24" rx="5" fill="url(#ij_grad)" />
          <rect x="9" y="9" width="18" height="18" rx="3" fill="#09090B" />
          <path d="M12 13V19M11 13H13M11 19H13" stroke="#FFFFFF" strokeWidth="1.8" strokeLinecap="round" />
          <path d="M16 13H18.5V17.5C18.5 18.6 17.7 19.2 16.5 19.2C15.8 19.2 15.2 18.9 14.9 18.4" stroke="#FFFFFF" strokeWidth="1.8" strokeLinecap="round" />
          <path d="M11.5 23H18.5" stroke="#FFFFFF" strokeWidth="2" strokeLinecap="round" />
          <defs>
            <linearGradient id="ij_grad" x1="6" y1="6" x2="30" y2="30" gradientUnits="userSpaceOnUse">
              <stop stopColor="#FE315D" />
              <stop offset="1" stopColor="#087CFA" />
            </linearGradient>
          </defs>
        </svg>
      );
    case 'webstorm':
      return (
        <svg width={size} height={size} viewBox="0 0 36 36" fill="none" className={`shrink-0 ${className}`}>
          <rect width="36" height="36" rx="8" fill="#0D2A38" />
          <rect x="6" y="6" width="24" height="24" rx="5" fill="#00CDD7" />
          <rect x="9" y="9" width="18" height="18" rx="3" fill="#09090B" />
          <text x="11" y="19" fill="#FFFFFF" fontSize="8" fontWeight="700" fontFamily="monospace">WS</text>
          <path d="M11.5 23H18.5" stroke="#FFFFFF" strokeWidth="2" strokeLinecap="round" />
        </svg>
      );
    case 'pycharm':
      return (
        <svg width={size} height={size} viewBox="0 0 36 36" fill="none" className={`shrink-0 ${className}`}>
          <rect width="36" height="36" rx="8" fill="#14291A" />
          <rect x="6" y="6" width="24" height="24" rx="5" fill="#21D789" />
          <rect x="9" y="9" width="18" height="18" rx="3" fill="#09090B" />
          <text x="11" y="19" fill="#FFFFFF" fontSize="8" fontWeight="700" fontFamily="monospace">PC</text>
          <path d="M11.5 23H18.5" stroke="#FFFFFF" strokeWidth="2" strokeLinecap="round" />
        </svg>
      );
    case 'sublime':
      return (
        <svg width={size} height={size} viewBox="0 0 36 36" fill="none" className={`shrink-0 ${className}`}>
          <rect width="36" height="36" rx="8" fill="#2D2D2D" />
          <path d="M10 13.5L26 8.5V14L10 19V13.5Z" fill="#FF9800" />
          <path d="M10 22L26 17V22.5L10 27.5V22Z" fill="#FF9800" />
          <path d="M10 19L26 22.5V17L10 13.5V19Z" fill="#FFB74D" />
        </svg>
      );
    case 'zed':
      return (
        <svg width={size} height={size} viewBox="0 0 36 36" fill="none" className={`shrink-0 ${className}`}>
          <rect width="36" height="36" rx="8" fill="#0F172A" />
          <path d="M10 11H26L14 25H26" stroke="#38BDF8" strokeWidth="2.8" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      );
    case 'windsurf':
      return (
        <svg width={size} height={size} viewBox="0 0 36 36" fill="none" className={`shrink-0 ${className}`}>
          <rect width="36" height="36" rx="8" fill="#062825" />
          <path d="M9 22C13 18 15 11 22 10C21 15 24 19 27 21C22 25 14 25 9 22Z" fill="#14B8A6" />
        </svg>
      );
    case 'xcode':
      return (
        <svg width={size} height={size} viewBox="0 0 36 36" fill="none" className={`shrink-0 ${className}`}>
          <rect width="36" height="36" rx="8" fill="#1D4ED8" />
          <path d="M11 25L24 12M14 11L25 22" stroke="#FFFFFF" strokeWidth="2.6" strokeLinecap="round" />
        </svg>
      );
    case 'obsidian':
      return (
        <svg width={size} height={size} viewBox="0 0 36 36" fill="none" className={`shrink-0 ${className}`}>
          <rect width="36" height="36" rx="8" fill="#2E1065" />
          <path d="M18 7L26 14L22 28L12 25L10 14L18 7Z" fill="#A855F7" />
          <path d="M18 7L26 14L18 20L18 7Z" fill="#C084FC" />
        </svg>
      );
    default: {
      const initial = (name || 'A').trim().charAt(0).toUpperCase();
      return (
        <div
          style={{ width: size, height: size }}
          className={`rounded-lg bg-neutral-100 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 text-neutral-700 dark:text-neutral-200 flex items-center justify-center font-semibold shrink-0 ${className}`}
        >
          {name ? (
            <span style={{ fontSize: Math.max(11, size * 0.44) }}>{initial}</span>
          ) : (
            <AppWindow style={{ width: size * 0.54, height: size * 0.54 }} />
          )}
        </div>
      );
    }
  }
};

export function getSlotIconProps(slot: ActiveSlot) {
  if (slot.type === 'builtin') {
    return { builtinAction: slot.action };
  }
  return {
    presetKey: slot.presetKey,
    customIconDataUrl: slot.customIconDataUrl,
    name: slot.name,
  };
}

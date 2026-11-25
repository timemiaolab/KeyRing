import React from 'react';
import { Minus, X } from 'lucide-react';
import { closeCurrentWindow, minimizeCurrentWindow } from '../tauriBridge';

interface TitleBarProps {
  title?: string;
}

export const TitleBar: React.FC<TitleBarProps> = ({ title = 'KeyRing' }) => {
  return (
    <div
      data-tauri-drag-region
      className="h-11 shrink-0 flex items-center px-3 border-b border-neutral-200/80 dark:border-neutral-800 bg-white dark:bg-[#1F1F1F] select-none"
    >
      <div data-tauri-drag-region className="flex-1 flex items-center min-w-0">
        <span
          data-tauri-drag-region
          className="text-[13px] font-medium text-neutral-800 dark:text-neutral-100 truncate"
        >
          {title}
        </span>
      </div>
      <div className="flex items-center gap-1">
        <button
          type="button"
          aria-label="最小化"
          onClick={() => void minimizeCurrentWindow()}
          className="w-7 h-7 rounded-md flex items-center justify-center text-neutral-500 hover:bg-neutral-100 dark:hover:bg-neutral-800 hover:text-neutral-800 dark:hover:text-neutral-100"
        >
          <Minus className="w-3.5 h-3.5" />
        </button>
        <button
          type="button"
          aria-label="关闭"
          onClick={() => void closeCurrentWindow()}
          className="w-7 h-7 rounded-md flex items-center justify-center text-neutral-500 hover:bg-red-500 hover:text-white"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
};

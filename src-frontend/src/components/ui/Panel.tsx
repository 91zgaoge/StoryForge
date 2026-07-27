import React, { useState } from 'react';
import { ChevronDown } from 'lucide-react';

interface PanelProps {
  title: string;
  children: React.ReactNode;
  collapsible?: boolean;
  defaultOpen?: boolean;
}

const headerClassName = [
  'flex items-center justify-between px-4 py-3',
  'border-b border-borderSubtle',
].join(' ');

export const Panel: React.FC<PanelProps> = ({
  title,
  children,
  collapsible,
  defaultOpen = true,
}) => {
  const [open, setOpen] = useState(defaultOpen);
  const contentId = React.useId();

  const headerContent = (
    <>
      <h3 className="text-xs font-bold uppercase tracking-wider text-cinema-gold/80">{title}</h3>
      {collapsible && (
        <ChevronDown
          aria-hidden="true"
          className={[
            'w-4 h-4 text-cinema-gold/80 transition-transform duration-300 ease-spring',
            open ? 'rotate-180' : '',
          ].join(' ')}
        />
      )}
    </>
  );

  return (
    <div className="bg-cinema-850 border border-borderSubtle rounded-panel shadow-panel overflow-hidden">
      {collapsible ? (
        <button
          type="button"
          aria-expanded={open}
          aria-controls={contentId}
          className={[
            headerClassName,
            'w-full text-left bg-transparent border-0 cursor-pointer hover:bg-cinema-800/50',
            'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cinema-gold/50',
          ].join(' ')}
          onClick={() => setOpen(v => !v)}
        >
          {headerContent}
        </button>
      ) : (
        <div className={headerClassName}>{headerContent}</div>
      )}
      <div
        id={contentId}
        className={[
          'transition-all duration-300 ease-spring overflow-hidden',
          open ? 'max-h-[1000px] opacity-100' : 'max-h-0 opacity-0',
        ].join(' ')}
      >
        <div className="p-4">{children}</div>
      </div>
    </div>
  );
};

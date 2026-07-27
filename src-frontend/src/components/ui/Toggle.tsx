import React from 'react';

interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: string;
}

export const Toggle: React.FC<ToggleProps> = ({ checked, onChange, label }) => {
  return (
    <label className="inline-flex items-center gap-3 cursor-pointer select-none">
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        onClick={() => onChange(!checked)}
        className={[
          'relative w-11 h-6 rounded-full transition-colors duration-200 ease-spring',
          'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cinema-gold/40',
          'active:scale-95',
          checked ? 'bg-cinema-gold' : 'bg-cinema-700',
        ].join(' ')}
      >
        <span
          className={[
            'absolute top-1 left-1 w-4 h-4 rounded-full bg-cinema-950 shadow-sm',
            'transition-transform duration-200 ease-spring',
            checked ? 'translate-x-5' : 'translate-x-0',
          ].join(' ')}
        />
      </button>
      {label && <span className="text-sm text-cinema-gold/90 font-medium">{label}</span>}
    </label>
  );
};

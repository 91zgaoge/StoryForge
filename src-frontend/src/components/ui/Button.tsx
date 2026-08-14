import React, { forwardRef } from 'react';

type ButtonVariant =
  | 'paper'
  | 'cinema'
  | 'cinema-outline'
  | 'primary'
  | 'secondary'
  | 'ghost'
  | 'danger';
type ButtonSize = 'sm' | 'md' | 'lg';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  isLoading?: boolean;
}

const variantMap: Record<ButtonVariant, string> = {
  paper:
    'bg-terracotta/15 text-terracotta-dark hover:bg-terracotta/25 focus-visible:ring-terracotta/40',
  cinema:
    'bg-cinema-gold/15 text-cinema-gold hover:bg-cinema-gold/25 focus-visible:ring-cinema-gold/40',
  'cinema-outline':
    'bg-transparent border border-cinema-600 text-cinema-gold hover:bg-cinema-800 focus-visible:ring-cinema-gold/30',
  // Legacy backstage variants mapped to new cinema styles
  primary:
    'bg-cinema-gold/15 text-cinema-gold hover:bg-cinema-gold/25 focus-visible:ring-cinema-gold/40',
  secondary:
    'bg-cinema-800 border border-cinema-700 text-cinema-100 hover:border-cinema-gold/50 hover:bg-cinema-700 focus-visible:ring-cinema-gold/30',
  ghost:
    'bg-transparent text-cinema-300 hover:text-cinema-50 hover:bg-cinema-800/50 focus-visible:ring-cinema-gold/30',
  danger:
    'bg-status-danger/10 border border-status-danger/30 text-status-danger hover:bg-status-danger/20 focus-visible:ring-status-danger/30',
};

const sizeMap: Record<ButtonSize, string> = {
  sm: 'px-3 py-1.5 text-xs rounded-panel',
  md: 'px-4 py-2 text-sm rounded-panel',
  lg: 'px-6 py-3 text-lg rounded-panel',
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  (
    { variant = 'paper', size = 'md', isLoading, children, disabled, className = '', ...props },
    ref
  ) => {
    const base =
      'inline-flex items-center justify-center font-medium transition-[background-color,color,border-color,transform,opacity] duration-300 ease-press enabled:active:scale-[0.98] motion-reduce:transition-none motion-reduce:active:scale-100 focus-visible:outline-none focus-visible:ring-2 disabled:opacity-50 disabled:cursor-not-allowed';

    return (
      <button
        ref={ref}
        className={[base, variantMap[variant], sizeMap[size], className].join(' ')}
        disabled={isLoading || disabled}
        {...props}
      >
        {isLoading && (
          <span className="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin mr-2" />
        )}
        {children}
      </button>
    );
  }
);

Button.displayName = 'Button';

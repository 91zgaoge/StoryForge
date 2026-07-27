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
  paper: 'bg-terracotta text-white hover:bg-terracotta-light focus-visible:ring-terracotta/40',
  cinema:
    'bg-cinema-gold text-cinema-950 hover:bg-cinema-gold-light focus-visible:ring-cinema-gold/40',
  'cinema-outline':
    'bg-transparent border border-cinema-600 text-cinema-gold hover:bg-cinema-800 focus-visible:ring-cinema-gold/30',
  // Legacy backstage variants mapped to new cinema styles
  primary:
    'bg-cinema-gold text-cinema-950 hover:bg-cinema-gold-light focus-visible:ring-cinema-gold/40',
  secondary:
    'bg-cinema-800 border border-cinema-700 text-gray-200 hover:border-cinema-gold/50 hover:bg-cinema-700 focus-visible:ring-cinema-gold/30',
  ghost:
    'bg-transparent text-gray-400 hover:text-white hover:bg-cinema-800/50 focus-visible:ring-cinema-gold/30',
  danger:
    'bg-red-500/10 border border-red-500/30 text-red-400 hover:bg-red-500/20 focus-visible:ring-red-500/30',
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
      'inline-flex items-center justify-center font-medium transition-all active:scale-95 focus-visible:outline-none focus-visible:ring-2 disabled:opacity-50 disabled:cursor-not-allowed';

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

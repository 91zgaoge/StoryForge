import React from 'react';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'paper' | 'cinema' | 'cinema-outline';
  size?: 'sm' | 'md';
}

export const Button: React.FC<ButtonProps> = ({
  variant = 'paper',
  size = 'md',
  className = '',
  children,
  ...props
}) => {
  const base =
    'inline-flex items-center justify-center font-medium transition-all active:scale-95 focus-visible:outline-none focus-visible:ring-2';

  const variants = {
    paper: 'bg-terracotta text-white hover:bg-terracotta-light focus-visible:ring-terracotta/40',
    cinema:
      'bg-cinema-gold text-cinema-950 hover:bg-cinema-gold-light focus-visible:ring-cinema-gold/40',
    'cinema-outline':
      'bg-transparent border border-cinema-600 text-cinema-gold hover:bg-cinema-800 focus-visible:ring-cinema-gold/30',
  };

  const sizes = {
    sm: 'px-3 py-1.5 text-xs rounded-panel',
    md: 'px-4 py-2 text-sm rounded-panel',
  };

  return (
    <button className={[base, variants[variant], sizes[size], className].join(' ')} {...props}>
      {children}
    </button>
  );
};

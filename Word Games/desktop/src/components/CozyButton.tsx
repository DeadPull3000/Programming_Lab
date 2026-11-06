import React from 'react';
import { sound } from '../services/sound';

interface CozyButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'coral' | 'mint' | 'yellow' | 'cream' | 'blue';
  size?: 'sm' | 'md' | 'lg';
  children: React.ReactNode;
}

export const CozyButton: React.FC<CozyButtonProps> = ({
  variant = 'cream',
  size = 'md',
  children,
  onClick,
  className = '',
  disabled,
  ...props
}) => {
  const getVariantStyles = () => {
    switch (variant) {
      case 'coral':
        return { background: 'var(--accent-coral)', color: '#fff' };
      case 'mint':
        return { background: 'var(--bg-mint)', color: 'var(--text-main)' };
      case 'yellow':
        return { background: 'var(--highlight-yellow)', color: '#4a3000' };
      case 'blue':
        return { background: 'var(--accent-blue)', color: 'var(--text-main)' };
      case 'cream':
      default:
        return { background: 'var(--panel-cream)', color: 'var(--text-main)' };
    }
  };

  const getSizeStyles = () => {
    switch (size) {
      case 'sm':
        return { padding: '6px 14px', fontSize: '0.9rem' };
      case 'lg':
        return { padding: '14px 28px', fontSize: '1.25rem' };
      case 'md':
      default:
        return { padding: '10px 22px', fontSize: '1.05rem' };
    }
  };

  const handleClick = (e: React.MouseEvent<HTMLButtonElement>) => {
    if (!disabled) {
      sound.playClick();
    }
    if (onClick) {
      onClick(e);
    }
  };

  return (
    <button
      {...props}
      disabled={disabled}
      onClick={handleClick}
      className={`cozy-btn ${className}`}
      style={{
        ...getVariantStyles(),
        ...getSizeStyles(),
      }}
    >
      {children}
    </button>
  );
};

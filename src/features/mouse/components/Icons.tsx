import type { ReactNode } from "react";

interface IconProps {
  size?: number;
}

interface BaseProps {
  size?: number;
  filled?: boolean;
  children: ReactNode;
}

function Svg({ size = 18, filled = false, children }: BaseProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill={filled ? "currentColor" : "none"}
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {children}
    </svg>
  );
}

export function SaveIcon({ size }: IconProps) {
  return (
    <Svg size={size}>
      <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
      <polyline points="17 21 17 13 7 13 7 21" />
      <polyline points="7 3 7 8 15 8" />
    </Svg>
  );
}

export function TrashIcon({ size }: IconProps) {
  return (
    <Svg size={size}>
      <polyline points="3 6 5 6 21 6" />
      <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
      <path d="M10 11v6" />
      <path d="M14 11v6" />
      <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
    </Svg>
  );
}

export function PlayIcon({ size }: IconProps) {
  return (
    <Svg size={size} filled>
      <polygon points="6 4 20 12 6 20 6 4" />
    </Svg>
  );
}

export function StopIcon({ size }: IconProps) {
  return (
    <Svg size={size} filled>
      <rect x="6" y="6" width="12" height="12" rx="2" />
    </Svg>
  );
}

export function EditIcon({ size }: IconProps) {
  return (
    <Svg size={size}>
      <path d="M17 3a2.83 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z" />
    </Svg>
  );
}

export function CloseIcon({ size }: IconProps) {
  return (
    <Svg size={size}>
      <line x1="18" y1="6" x2="6" y2="18" />
      <line x1="6" y1="6" x2="18" y2="18" />
    </Svg>
  );
}

export function LockIcon({ size }: IconProps) {
  return (
    <Svg size={size}>
      <rect x="3" y="11" width="18" height="11" rx="2" />
      <path d="M7 11V7a5 5 0 0 1 10 0v4" />
    </Svg>
  );
}

export function ChevronIcon({ size, open }: IconProps & { open: boolean }) {
  return (
    <Svg size={size}>
      {open ? <polyline points="6 9 12 15 18 9" /> : <polyline points="9 18 15 12 9 6" />}
    </Svg>
  );
}

export function RecordIcon({ size }: IconProps) {
  return (
    <Svg size={size} filled>
      <circle cx="12" cy="12" r="8" />
    </Svg>
  );
}

export function MouseIcon({ size }: IconProps) {
  return (
    <Svg size={size}>
      <rect x="6" y="3" width="12" height="18" rx="6" />
      <path d="M12 7v4" />
    </Svg>
  );
}
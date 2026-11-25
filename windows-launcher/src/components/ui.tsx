import type { ReactNode } from "react";
import type { LucideIcon } from "lucide-react";

export function PageHeader({ icon: Icon, title, subtitle, actions }: { icon: LucideIcon; title: string; subtitle?: ReactNode; actions?: ReactNode }) {
  return (
    <header className="page-header">
      <div className="page-title">
        <span className="page-icon">
          <Icon size={22} />
        </span>
        <div>
          <h1>{title}</h1>
          {subtitle && <p className="muted">{subtitle}</p>}
        </div>
      </div>
      {actions && <div className="actions">{actions}</div>}
    </header>
  );
}

export function Meter({ value, tone }: { value: number; tone?: "warn" | "danger" }) {
  const clamped = Math.max(0, Math.min(100, value));
  return (
    <div className={`bar ${tone ?? (clamped > 90 ? "danger" : clamped > 75 ? "warn" : "")}`}>
      <span style={{ width: `${clamped}%` }} />
    </div>
  );
}

export function ScoreRing({ value, size = 132, label }: { value: number; size?: number; label?: string }) {
  const stroke = 10;
  const radius = (size - stroke) / 2;
  const circumference = 2 * Math.PI * radius;
  return (
    <div className="ring" style={{ width: size, height: size }}>
      <svg viewBox={`0 0 ${size} ${size}`}>
        <defs>
          <linearGradient id="ring-gradient" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0" stopColor="var(--accent)" />
            <stop offset="1" stopColor="var(--accent-2)" />
          </linearGradient>
        </defs>
        <circle cx={size / 2} cy={size / 2} r={radius} className="ring-track" strokeWidth={stroke} />
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          className="ring-value"
          strokeWidth={stroke}
          strokeDasharray={circumference}
          strokeDashoffset={circumference * (1 - value / 100)}
        />
      </svg>
      <div className="ring-label">
        <strong>{value}</strong>
        {label && <span>{label}</span>}
      </div>
    </div>
  );
}

export function Skeleton({ height = 120 }: { height?: number }) {
  return <div className="skeleton" style={{ height }} />;
}

export function Spec({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="spec">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

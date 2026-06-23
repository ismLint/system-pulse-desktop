import { clsx } from 'clsx'
// @ts-ignore
import s from './ui.module.css'

export function Card({ children, className }: { children: React.ReactNode; className?: string }) {
  return <div className={clsx(s.card, className)}>{children}</div>
}

export function PageHeader({ title, subtitle, action }: { title: string; subtitle?: string; action?: React.ReactNode }) {
  return (
    <div className={s.pageHeader}>
      <div>
        <h1 className={s.pageTitle}>{title}</h1>
        {subtitle && <p className={s.pageSub}>{subtitle}</p>}
      </div>
      {action}
    </div>
  )
}

type Variant = 'primary' | 'ghost' | 'danger' | 'success'
type Size = 'sm' | 'md' | 'lg'

export function Button({ children, variant = 'primary', size = 'md', onClick, disabled, loading, type = 'button', className }: {
  children: React.ReactNode; variant?: Variant; size?: Size
  onClick?: () => void; disabled?: boolean; loading?: boolean
  type?: 'button' | 'submit'; className?: string
}) {
  return (
    <button type={type} onClick={onClick} disabled={disabled || loading}
      className={clsx(s.btn, s[`btn_${variant}`], s[`btn_${size}`], className)}>
      {loading ? <span className={s.spinner} /> : children}
    </button>
  )
}

export function Badge({ children, color = 'blue' }: {
  children: React.ReactNode; color?: 'blue' | 'green' | 'yellow' | 'red' | 'purple'
}) {
  return <span className={clsx(s.badge, s[`badge_${color}`])}>{children}</span>
}

export function StatCard({ label, value, icon, color = 'var(--accent)', sublabel }: {
  label: string; value: string; icon: React.ReactNode; color?: string; sublabel?: string
}) {
  return (
    <div className={s.statCard}>
      <div className={s.statIcon} style={{ color, background: `color-mix(in srgb, ${color} 14%, transparent)` }}>
        {icon}
      </div>
      <div className={s.statBody}>
        <span className={s.statLabel}>{label}</span>
        <span className={s.statValue}>{value}</span>
        {sublabel && <span className={s.statSub}>{sublabel}</span>}
      </div>
    </div>
  )
}

export function Spinner({ size = 24 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" className={s.spinSvg}>
      <circle cx="12" cy="12" r="10" stroke="var(--border-bright)" strokeWidth="2" />
      <path d="M12 2a10 10 0 0 1 10 10" stroke="var(--accent)" strokeWidth="2" strokeLinecap="round" />
    </svg>
  )
}

export function EmptyState({ icon, title, description, action }: {
  icon: React.ReactNode; title: string; description?: string; action?: React.ReactNode
}) {
  return (
    <div className={s.empty}>
      <div className={s.emptyIcon}>{icon}</div>
      <h3 className={s.emptyTitle}>{title}</h3>
      {description && <p className={s.emptyDesc}>{description}</p>}
      {action}
    </div>
  )
}

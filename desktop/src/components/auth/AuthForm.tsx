import { useState } from 'react'
import { Activity, Eye, EyeOff, AlertCircle } from 'lucide-react'
// @ts-ignore
import styles from './AuthForm.module.css'

interface Field {
  name: string; label: string; type?: string
  placeholder?: string; autoComplete?: string
}

interface Props {
  title: string; subtitle?: string; fields: Field[]
  submitLabel: string; loading?: boolean; error?: string | null
  onSubmit: (v: Record<string, string>) => void; footer?: React.ReactNode
}

export default function AuthForm({ title, subtitle, fields, submitLabel, loading, error, onSubmit, footer }: Props) {
  const [values, setValues] = useState<Record<string, string>>({})
  const [showPw, setShowPw] = useState<Record<string, boolean>>({})

  const set = (name: string) => (e: React.ChangeEvent<HTMLInputElement>) =>
    setValues(v => ({ ...v, [name]: e.target.value }))

  const handleSubmit = (e: React.FormEvent) => { e.preventDefault(); onSubmit(values) }

  return (
    <div className={styles.page}>
      <div className={styles.bg}><div className={styles.grid} /><div className={styles.orb} /></div>
      <div className={styles.card}>
        <div className={styles.logo}>
          <Activity size={18} className={styles.logoIcon} />
          <span>System<strong>Pulse</strong></span>
        </div>

        <h1 className={styles.title}>{title}</h1>
        {subtitle && <p className={styles.sub}>{subtitle}</p>}

        {error && (
          <div className={styles.err}>
            <AlertCircle size={14} /><span>{error}</span>
          </div>
        )}

        <form onSubmit={handleSubmit} className={styles.form}>
          {fields.map(f => {
            const isPw = f.type === 'password'
            const shown = showPw[f.name]
            return (
              <div key={f.name} className={styles.field}>
                <label className={styles.label}>{f.label}</label>
                <div className={styles.inputWrap}>
                  <input
                    type={isPw ? (shown ? 'text' : 'password') : (f.type ?? 'text')}
                    placeholder={f.placeholder}
                    autoComplete={f.autoComplete}
                    value={values[f.name] ?? ''}
                    onChange={set(f.name)}
                    required
                  />
                  {isPw && (
                    <button type="button" className={styles.eyeBtn}
                      onClick={() => setShowPw(p => ({ ...p, [f.name]: !p[f.name] }))}>
                      {shown ? <EyeOff size={14} /> : <Eye size={14} />}
                    </button>
                  )}
                </div>
              </div>
            )
          })}
          <button type="submit" className={styles.submit} disabled={loading}>
            {loading ? <span className={styles.spinner} /> : submitLabel}
          </button>
        </form>

        {footer && <div className={styles.footer}>{footer}</div>}
      </div>
    </div>
  )
}

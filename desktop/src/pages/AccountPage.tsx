import { useState } from 'react'
import { useAuthStore } from '@/store/authStore'
import { invoke, extractMessage } from '@/utils/invoke'
import { PageHeader, Card, Button, Badge } from '@/components/ui'
import { KeyRound, Mail, AtSign, CheckCircle, User } from 'lucide-react'
// @ts-ignore
import styles from './AccountPage.module.css'

export default function AccountPage() {
  const { user, setUser } = useAuthStore()
  const [panel, setPanel] = useState<'password' | 'email' | 'login' | null>(null)

  return (
    <div>
      <PageHeader title="My Account" subtitle="Profile & security settings" />

      <Card className={styles.profile}>
        <div className={styles.avatarRow}>
          <div className={styles.avatar}>
            {user?.first_name?.[0] ?? user?.login?.[0]?.toUpperCase() ?? 'U'}
          </div>
          <div>
            <p className={styles.fullName}>
              {user?.first_name ? `${user.first_name} ${user.last_name ?? ''}`.trim() : user?.login}
            </p>
            <Badge color={user?.subscription === 'premium' ? 'purple' : 'blue'}>
              {user?.subscription ?? 'free'}
            </Badge>
          </div>
        </div>

        <div className={styles.fields}>
          <Field icon={<AtSign size={14} />} label="Login"      value={user?.login ?? ''} />
          <Field icon={<Mail size={14} />}   label="Email"      value={user?.email ?? ''} />
          <Field icon={<User size={14} />}   label="First name" value={user?.first_name ?? '—'} />
          <Field icon={<User size={14} />}   label="Last name"  value={user?.last_name ?? '—'} />
        </div>

        <div className={styles.actions}>
          <Button variant="ghost" size="sm" onClick={() => setPanel(p => p === 'password' ? null : 'password')}>
            <KeyRound size={13} /> Change password
          </Button>
          <Button variant="ghost" size="sm" onClick={() => setPanel(p => p === 'email' ? null : 'email')}>
            <Mail size={13} /> Change email
          </Button>
          <Button variant="ghost" size="sm" onClick={() => setPanel(p => p === 'login' ? null : 'login')}>
            <AtSign size={13} /> Change login
          </Button>
        </div>
      </Card>

      {panel === 'password' && (
        <ChangePasswordForm onDone={() => setPanel(null)} />
      )}
      {panel === 'email' && (
        <ChangeEmailForm onDone={() => setPanel(null)} onSuccess={email => setUser({ ...user!, email })} />
      )}
      {panel === 'login' && (
        <ChangeLoginForm onDone={() => setPanel(null)} onSuccess={login => setUser({ ...user!, login })} />
      )}
    </div>
  )
}

function Field({ icon, label, value }: { icon: React.ReactNode; label: string; value: string }) {
  return (
    <div className={styles.field}>
      <span className={styles.fieldIcon}>{icon}</span>
      <div>
        <span className={styles.fieldLabel}>{label}</span>
        <span className={styles.fieldValue}>{value}</span>
      </div>
    </div>
  )
}

function ChangePasswordForm({ onDone }: { onDone: () => void }) {
  const [f, setF] = useState({ current_password: '', new_password: '', confirm_password: '' })
  const [loading, setLoading] = useState(false)
  const [err, setErr] = useState<string | null>(null)
  const [ok, setOk] = useState(false)
  const set = (k: keyof typeof f) => (e: React.ChangeEvent<HTMLInputElement>) => setF(p => ({ ...p, [k]: e.target.value }))

  const submit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (f.new_password !== f.confirm_password) return setErr('Passwords do not match')
    setLoading(true); setErr(null)
    try {
      await invoke('change_password', { input: f })
      setOk(true); setTimeout(onDone, 1400)
    } catch (e) { setErr(extractMessage(e)) }
    finally { setLoading(false) }
  }

  return (
    <Card className={styles.subPanel}>
      <h3 className={styles.subTitle}><KeyRound size={14} /> Change Password</h3>
      {ok ? <div className={styles.success}><CheckCircle size={14} /> Password updated!</div> : (
        <form onSubmit={submit} className={styles.subForm}>
          {err && <div className={styles.subErr}>{err}</div>}
          <input type="password" placeholder="Current password" value={f.current_password} onChange={set('current_password')} required />
          <input type="password" placeholder="New password (min 8)" value={f.new_password} onChange={set('new_password')} required />
          <input type="password" placeholder="Confirm new password" value={f.confirm_password} onChange={set('confirm_password')} required />
          <div className={styles.subBtns}>
            <Button variant="ghost" size="sm" type="button" onClick={onDone}>Cancel</Button>
            <Button size="sm" type="submit" loading={loading}>Update</Button>
          </div>
        </form>
      )}
    </Card>
  )
}

function ChangeEmailForm({ onDone, onSuccess }: { onDone: () => void; onSuccess: (e: string) => void }) {
  const [email, setEmail] = useState('')
  const [loading, setLoading] = useState(false)
  const [err, setErr] = useState<string | null>(null)
  const [ok, setOk] = useState(false)

  const submit = async (e: React.FormEvent) => {
    e.preventDefault(); setLoading(true); setErr(null)
    try {
      await invoke('change_email', { input: { new_email: email } })
      onSuccess(email); setOk(true); setTimeout(onDone, 1400)
    } catch (e) { setErr(extractMessage(e)) }
    finally { setLoading(false) }
  }

  return (
    <Card className={styles.subPanel}>
      <h3 className={styles.subTitle}><Mail size={14} /> Change Email</h3>
      {ok ? <div className={styles.success}><CheckCircle size={14} /> Email updated!</div> : (
        <form onSubmit={submit} className={styles.subForm}>
          {err && <div className={styles.subErr}>{err}</div>}
          <input type="email" placeholder="New email address" value={email} onChange={e => setEmail(e.target.value)} required />
          <div className={styles.subBtns}>
            <Button variant="ghost" size="sm" type="button" onClick={onDone}>Cancel</Button>
            <Button size="sm" type="submit" loading={loading}>Update</Button>
          </div>
        </form>
      )}
    </Card>
  )
}

function ChangeLoginForm({ onDone, onSuccess }: { onDone: () => void; onSuccess: (l: string) => void }) {
  const [login, setLogin] = useState('')
  const [loading, setLoading] = useState(false)
  const [err, setErr] = useState<string | null>(null)
  const [ok, setOk] = useState(false)

  const submit = async (e: React.FormEvent) => {
    e.preventDefault(); setLoading(true); setErr(null)
    try {
      await invoke('change_login', { input: { new_login: login } })
      onSuccess(login); setOk(true); setTimeout(onDone, 1400)
    } catch (e) { setErr(extractMessage(e)) }
    finally { setLoading(false) }
  }

  return (
    <Card className={styles.subPanel}>
      <h3 className={styles.subTitle}><AtSign size={14} /> Change Login</h3>
      {ok ? <div className={styles.success}><CheckCircle size={14} /> Login updated!</div> : (
        <form onSubmit={submit} className={styles.subForm}>
          {err && <div className={styles.subErr}>{err}</div>}
          <input type="text" placeholder="New login (3+ chars)" value={login} onChange={e => setLogin(e.target.value)} required />
          <div className={styles.subBtns}>
            <Button variant="ghost" size="sm" type="button" onClick={onDone}>Cancel</Button>
            <Button size="sm" type="submit" loading={loading}>Update</Button>
          </div>
        </form>
      )}
    </Card>
  )
}

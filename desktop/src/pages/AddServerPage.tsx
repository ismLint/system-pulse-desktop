import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Server, Plug, Monitor, Wifi } from 'lucide-react'
import { PageHeader, Button, Card } from '@/components/ui'
import { invoke, extractMessage } from '@/utils/invoke'
import type { Server as ServerType } from '@/types'
// @ts-ignore
import styles from './AddServerPage.module.css'

type Mode = 'local' | 'remote'

export default function AddServerPage() {
  const navigate = useNavigate()
  const [mode, setMode]     = useState<Mode>('remote')
  const [name, setName]     = useState('')
  const [host, setHost]     = useState('')
  const [user, setUser]     = useState('')
  const [pass, setPass]     = useState('')
  const [desc, setDesc]     = useState('')
  const [loading, setLoading]   = useState(false)
  const [testing, setTesting]   = useState(false)
  const [testResult, setTestResult] = useState<{ ok: boolean; msg: string } | null>(null)
  const [error, setError]   = useState<string | null>(null)

  const handleModeChange = (m: Mode) => {
    setMode(m)
    setTestResult(null)
    if (m === 'local') {
      setHost('localhost')
      setUser('local')
      setPass('')
    } else {
      setHost('')
      setUser('')
      setPass('')
    }
  }

  const handleTest = async () => {
    if (mode === 'local') {
      setTestResult({ ok: true, msg: 'Local machine — metrics via /proc' })
      return
    }
    setTesting(true); setTestResult(null)
    try {
      // Create temporary server to test, then delete
      const s = await invoke<ServerType>('create_server', {
        input: { name: `__test_${Date.now()}`, host, ssh_user: user, password: pass, description: null, server_type: 'remote' },
      })
      try {
        const msg = await invoke<string>('test_connection', { id: s.id })
        setTestResult({ ok: true, msg })
      } finally {
        await invoke('delete_server', { id: s.id }).catch(() => {})
      }
    } catch (e) {
      setTestResult({ ok: false, msg: extractMessage(e) })
    } finally { setTesting(false) }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault(); setLoading(true); setError(null)
    try {
      const s = await invoke<ServerType>('create_server', {
        input: {
          name: name.trim(),
          host: mode === 'local' ? 'localhost' : host.trim(),
          ssh_user: mode === 'local' ? 'local' : user.trim(),
          password: mode === 'local' ? '' : pass,
          description: desc.trim() || null,
          server_type: mode,
        },
      })
      navigate(`/servers/${s.id}`)
    } catch (err) { setError(extractMessage(err)) }
    finally { setLoading(false) }
  }

  return (
    <div>
      <PageHeader title="Add Server" subtitle="Connect a server to monitor" />

      <div className={styles.layout}>
        <Card className={styles.formCard}>
          {/* Mode toggle */}
          <div className={styles.modeToggle}>
            <button
              type="button"
              className={`${styles.modeBtn} ${mode === 'local' ? styles.modeBtnActive : ''}`}
              onClick={() => handleModeChange('local')}
            >
              <Monitor size={15} />
              <span>Local machine</span>
              <span className={styles.modeSub}>this computer</span>
            </button>
            <button
              type="button"
              className={`${styles.modeBtn} ${mode === 'remote' ? styles.modeBtnActive : ''}`}
              onClick={() => handleModeChange('remote')}
            >
              <Wifi size={15} />
              <span>Remote server</span>
              <span className={styles.modeSub}>via SSH</span>
            </button>
          </div>

          <form onSubmit={handleSubmit} className={styles.form}>
            {error && <div className={styles.err}>{error}</div>}

            <div className={styles.field}>
              <label>Server Name</label>
              <input
                value={name}
                onChange={e => setName(e.target.value)}
                placeholder={mode === 'local' ? 'My Computer' : 'Production API'}
                required
              />
            </div>

            {mode === 'remote' ? (
              <>
                <div className={styles.row}>
                  <div className={styles.field}>
                    <label>Host / IP</label>
                    <input value={host} onChange={e => setHost(e.target.value)}
                      placeholder="192.168.1.100" required />
                  </div>
                  <div className={styles.field}>
                    <label>SSH User</label>
                    <input value={user} onChange={e => setUser(e.target.value)}
                      placeholder="root" required />
                  </div>
                </div>
                <div className={styles.field}>
                  <label>SSH Password</label>
                  <input type="password" value={pass} onChange={e => setPass(e.target.value)}
                    placeholder="••••••••" required />
                </div>
              </>
            ) : (
              <div className={styles.localInfo}>
                <Monitor size={16} className={styles.localIcon} />
                <div>
                  <p className={styles.localTitle}>Local collection via <code>/proc</code> &amp; <code>/sys</code></p>
                  <p className={styles.localDesc}>No SSH needed. Metrics are read directly from the Linux kernel. Requires the app to run on Linux.</p>
                </div>
              </div>
            )}

            <div className={styles.field}>
              <label>Description <span className={styles.opt}>(optional)</span></label>
              <input value={desc} onChange={e => setDesc(e.target.value)}
                placeholder={mode === 'local' ? 'My workstation' : 'Frankfurt DC, prod'} />
            </div>

            {testResult && (
              <div className={testResult.ok ? styles.testOk : styles.testFail}>
                {testResult.ok ? `✓ ${testResult.msg}` : `✗ ${testResult.msg}`}
              </div>
            )}

            <div className={styles.actions}>
              <Button variant="ghost" size="sm" type="button" onClick={() => navigate('/servers')}>
                Cancel
              </Button>
              <div className={styles.rightBtns}>
                {mode === 'remote' && (
                  <Button variant="ghost" size="sm" type="button" loading={testing}
                    onClick={handleTest} disabled={!host || !user || !pass}>
                    <Plug size={13} /> Test SSH
                  </Button>
                )}
                {mode === 'local' && (
                  <Button variant="ghost" size="sm" type="button" onClick={handleTest}>
                    <Plug size={13} /> Verify local
                  </Button>
                )}
                <Button type="submit" size="sm" loading={loading}>
                  <Server size={13} /> Add server
                </Button>
              </div>
            </div>
          </form>
        </Card>

        <div className={styles.hint}>
          {mode === 'remote' ? (
            <>
              <h3>Remote requirements</h3>
              <ul>
                <li>Linux server with SSH access</li>
                <li><code>sshpass</code> on this machine</li>
                <li>Standard tools: <code>top</code>, <code>free</code>, <code>df</code></li>
              </ul>
              <h3 style={{ marginTop: '1rem' }}>Install sshpass</h3>
              <div className={styles.code}><code>choco install sshpass</code></div>
            </>
          ) : (
            <>
              <h3>Local requirements</h3>
              <ul>
                <li>App must run on Linux</li>
                <li>Read access to <code>/proc</code> and <code>/sys</code></li>
                <li>No additional tools needed</li>
              </ul>
              <div className={styles.code}>
                <code>Metrics: CPU · RAM · Disk · Net · Temp · Load</code>
              </div>
            </>
          )}
          <p className={styles.note}>
            Metrics are collected every 5 seconds and stored locally in SQLite.
          </p>
        </div>
      </div>
    </div>
  )
}

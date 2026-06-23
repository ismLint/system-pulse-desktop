import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { Server, Activity, ChevronRight } from 'lucide-react'
import { PageHeader, StatCard, Badge, Spinner, EmptyState } from '@/components/ui'
import { useServers } from '@/hooks/useServers'
import { useAuthStore } from '@/store/authStore'
import { invoke } from '@/utils/invoke'
import { formatPct, formatBytes } from '@/utils/format'
import type { Metric } from '@/types'
// @ts-ignore
import styles from './DashboardPage.module.css'

export default function DashboardPage() {
  const { user } = useAuthStore()
  const { servers, loading } = useServers()
  const [latest, setLatest] = useState<Record<string, Metric>>({})

  useEffect(() => {
    servers.forEach(s => {
      invoke<Metric | null>('get_latest_metric', { serverId: s.id })
        .then(m => { if (m) setLatest(p => ({ ...p, [s.id]: m })) })
        .catch(() => {})
    })
  }, [servers])

  const hour = new Date().getHours()
  const greet = hour < 12 ? 'Good morning' : hour < 18 ? 'Good afternoon' : 'Good evening'

  return (
    <div>
      <PageHeader
        title={`${greet}, ${user?.first_name ?? user?.login ?? 'User'}`}
        subtitle="Infrastructure overview"
      />

      {loading ? (
        <div className={styles.center}><Spinner size={28} /></div>
      ) : servers.length === 0 ? (
        <EmptyState
          icon={<Server size={24} />}
          title="No servers connected"
          description="Add your first server to start monitoring."
          action={<Link to="/servers/add"><button className={styles.addBtn}>Add server</button></Link>}
        />
      ) : (
        <>
          <div className={styles.statsRow}>
            <StatCard label="Total Servers" value={String(servers.length)}
              icon={<Server size={18} />} color="var(--accent)" />
            <StatCard label="Active" value={String(servers.filter(s => s.is_active).length)}
              icon={<Activity size={18} />} color="var(--success)" />
          </div>

          <p className={styles.sectionLabel}>Servers</p>
          <div className={styles.cards}>
            {servers.map(s => {
              const m = latest[s.id]
              return (
                <Link key={s.id} to={`/servers/${s.id}`} className={styles.card}>
                  <div className={styles.cardTop}>
                    <div className={styles.cardIcon}><Server size={16} /></div>
                    <div className={styles.cardMeta}>
                      <span className={styles.cardName}>{s.name}</span>
                      <span className={styles.cardHost}>{s.host}</span>
                    </div>
                    <Badge color={s.is_active ? 'green' : 'red'}>
                      {s.is_active ? 'active' : 'off'}
                    </Badge>
                  </div>
                  {m ? (
                    <div className={styles.mini}>
                      <MiniStat label="CPU"  value={formatPct(m.cpu_usage)}    color="var(--chart-cpu)" />
                      <MiniStat label="RAM"  value={formatPct(m.ram_usage_pct)} color="var(--chart-ram)" />
                      {m.temperature_cpu != null &&
                        <MiniStat label="Temp" value={`${m.temperature_cpu.toFixed(0)}°C`} color="var(--chart-temp)" />}
                      <MiniStat label="Total" value={formatBytes(m.ram_total)} color="var(--text-muted)" />
                    </div>
                  ) : (
                    <p className={styles.noData}>Waiting for metrics…</p>
                  )}
                  <div className={styles.cardFooter}>
                    <span>View details</span><ChevronRight size={13} />
                  </div>
                </Link>
              )
            })}
          </div>
        </>
      )}
    </div>
  )
}

function MiniStat({ label, value, color }: { label: string; value: string; color: string }) {
  return (
    <div className={styles.miniStat}>
      <span className={styles.miniLabel}>{label}</span>
      <span className={styles.miniValue} style={{ color }}>{value}</span>
    </div>
  )
}

import { useParams, useNavigate } from 'react-router-dom'
import { useEffect, useState } from 'react'
import { ArrowLeft, Cpu, MemoryStick, Thermometer, HardDrive, RefreshCw, Wifi, WifiOff } from 'lucide-react'
import { PageHeader, StatCard, Spinner, Badge } from '@/components/ui'
import MetricChart from '@/components/charts/MetricChart'
import { useMetrics } from '@/hooks/useMetrics'
import { invoke } from '@/utils/invoke'
import { formatBytes, formatPct, formatTemp, formatUptime, formatTime } from '@/utils/format'
import type { Server } from '@/types'
// @ts-ignore
import styles from './ServerDetailPage.module.css'

export default function ServerDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [server, setServer] = useState<Server | null>(null)
  const { metrics, latest, loading, polling } = useMetrics(id, 120)

  useEffect(() => {
    if (!id) return
    invoke<Server>('get_server', { id })
      .then(setServer)
      .catch(() => navigate('/servers'))
  }, [id, navigate])

  const chart = metrics.map(m => ({
    time:   formatTime(m.collected_at),
    cpu:    m.cpu_usage,
    ram:    m.ram_usage_pct,
    temp:   m.temperature_cpu ?? 0,
    disk_r: (m.disk_read_bytes_sec ?? 0) / 1024 / 1024,
    disk_w: (m.disk_write_bytes_sec ?? 0) / 1024 / 1024,
    net_rx: (m.net_rx_bytes_sec ?? 0) / 1024,
    net_tx: (m.net_tx_bytes_sec ?? 0) / 1024,
    load:   m.load_avg_1 ?? 0,
  }))

  return (
    <div>
      <div className={styles.back}>
        <button onClick={() => navigate('/servers')} className={styles.backBtn}>
          <ArrowLeft size={14} /> Servers
        </button>
      </div>

      <PageHeader
        title={server?.name ?? '…'}
        subtitle={server ? (server.server_type === 'local' ? 'localhost · local machine' : `${server.ssh_user}@${server.host}`) : ''}
        action={
          polling
            ? <Badge color="green"><Wifi size={9} style={{ marginRight: 3 }} />Live</Badge>
            : <Badge color="yellow"><WifiOff size={9} style={{ marginRight: 3 }} />Connecting…</Badge>
        }
      />

      {loading ? (
        <div className={styles.center}><Spinner size={28} /></div>
      ) : (
        <>
          <div className={styles.stats}>
            <StatCard label="CPU" value={latest ? formatPct(latest.cpu_usage) : '—'}
              icon={<Cpu size={18} />} color="var(--chart-cpu)"
              sublabel={latest?.cpu_cores ? `${latest.cpu_cores} cores` : undefined} />
            <StatCard label="RAM" value={latest ? formatPct(latest.ram_usage_pct) : '—'}
              icon={<MemoryStick size={18} />} color="var(--chart-ram)"
              sublabel={latest ? `${formatBytes(latest.ram_used)} / ${formatBytes(latest.ram_total)}` : undefined} />
            <StatCard label="Temp" value={latest?.temperature_cpu ? formatTemp(latest.temperature_cpu) : 'N/A'}
              icon={<Thermometer size={18} />} color="var(--chart-temp)" />
            <StatCard label="Uptime" value={latest?.uptime_seconds ? formatUptime(latest.uptime_seconds) : '—'}
              icon={<RefreshCw size={18} />} color="var(--chart-net)"
              sublabel={latest?.load_avg_1 ? `load: ${latest.load_avg_1.toFixed(2)}` : undefined} />
            {latest?.disk_total && (
              <StatCard label="Disk"
                value={formatPct(((latest.disk_used ?? 0) / latest.disk_total) * 100)}
                icon={<HardDrive size={18} />} color="var(--chart-disk)"
                sublabel={`${formatBytes(latest.disk_used ?? 0)} / ${formatBytes(latest.disk_total)}`} />
            )}
          </div>

          <div className={styles.charts}>
            <MetricChart title="CPU Usage"   data={chart} dataKey="cpu"    color="var(--chart-cpu)"  unit="%" domain={[0,100]} referenceValue={85} />
            <MetricChart title="RAM Usage"   data={chart} dataKey="ram"    color="var(--chart-ram)"  unit="%" domain={[0,100]} />
            {chart.some(d => d.temp > 0) &&
              <MetricChart title="CPU Temp"  data={chart} dataKey="temp"   color="var(--chart-temp)" unit="°C" domain={[0,100]} referenceValue={80} formatValue={v => v.toFixed(1)} />}
            <MetricChart title="Disk Read"   data={chart} dataKey="disk_r" color="var(--chart-disk)" unit=" MB/s" domain={[0,'auto']} formatValue={v => v.toFixed(2)} />
            <MetricChart title="Disk Write"  data={chart} dataKey="disk_w" color="#c084fc"            unit=" MB/s" domain={[0,'auto']} formatValue={v => v.toFixed(2)} />
            <MetricChart title="Net RX"      data={chart} dataKey="net_rx" color="var(--chart-net)"  unit=" KB/s" domain={[0,'auto']} formatValue={v => v.toFixed(1)} />
            <MetricChart title="Net TX"      data={chart} dataKey="net_tx" color="#22d3ee"            unit=" KB/s" domain={[0,'auto']} formatValue={v => v.toFixed(1)} />
            <MetricChart title="Load Avg 1m" data={chart} dataKey="load"   color="#f472b6"            unit=""      domain={[0,'auto']} formatValue={v => v.toFixed(2)} />
          </div>
        </>
      )}
    </div>
  )
}

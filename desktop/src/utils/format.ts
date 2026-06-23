export function formatBytes(b: number, d = 1): string {
  if (b === 0) return '0 B'
  const k = 1024, s = ['B','KB','MB','GB','TB']
  const i = Math.floor(Math.log(b) / Math.log(k))
  return `${(b / k ** i).toFixed(d)} ${s[i]}`
}
export function formatUptime(s: number): string {
  const d = Math.floor(s / 86400), h = Math.floor((s % 86400) / 3600), m = Math.floor((s % 3600) / 60)
  if (d > 0) return `${d}d ${h}h ${m}m`
  if (h > 0) return `${h}h ${m}m`
  return `${m}m`
}
export function formatPct(v: number, d = 1) { return `${v.toFixed(d)}%` }
export function formatTemp(c: number) { return `${c.toFixed(1)}°C` }
export function formatTime(iso: string): string {
  return new Date(iso).toLocaleTimeString('en-US', { hour12: false })
}

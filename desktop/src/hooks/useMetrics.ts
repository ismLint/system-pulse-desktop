import { useState, useEffect, useCallback, useRef } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke, extractMessage } from '@/utils/invoke'
import type { Metric } from '@/types'

export function useMetrics(serverId: string | undefined, limit = 120) {
  const [metrics, setMetrics] = useState<Metric[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [polling, setPolling] = useState(false)
  const unlistenRef = useRef<(() => void) | null>(null)

  const fetchHistory = useCallback(async () => {
    if (!serverId) return
    setLoading(true)
    try {
      const data = await invoke<Metric[]>('get_metrics', { serverId, limit })
      setMetrics(data)
      setError(null)
    } catch (e) {
      setError(extractMessage(e))
    } finally {
      setLoading(false)
    }
  }, [serverId, limit])

  const startPolling = useCallback(async () => {
    if (!serverId) return
    try {
      // Listen for Tauri events from background polling task
      const unlisten = await listen<Metric>(`metric:${serverId}`, (event) => {
        setMetrics(prev => {
          const next = [...prev, event.payload]
          return next.length > 300 ? next.slice(-300) : next
        })
        setError(null)
      })
      unlistenRef.current = unlisten

      await listen<{ error: string }>(`metric_error:${serverId}`, (event) => {
        setError(event.payload.error)
      })

      await invoke('start_polling', { serverId, intervalSecs: 5 })
      setPolling(true)
    } catch (e) {
      setError(extractMessage(e))
    }
  }, [serverId])

  const stopPolling = useCallback(async () => {
    if (serverId) await invoke('stop_polling', { serverId }).catch(() => {})
    unlistenRef.current?.()
    unlistenRef.current = null
    setPolling(false)
  }, [serverId])

  useEffect(() => {
    if (!serverId) return
    fetchHistory().then(() => startPolling())
    return () => { stopPolling() }
  }, [serverId])

  const latest = metrics.length > 0 ? metrics[metrics.length - 1] : null

  return { metrics, latest, loading, error, polling, refetch: fetchHistory }
}

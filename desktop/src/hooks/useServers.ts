import { useState, useEffect, useCallback } from 'react'
import { invoke, extractMessage } from '@/utils/invoke'
import type { Server } from '@/types'

export function useServers() {
  const [servers, setServers] = useState<Server[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetch = useCallback(async () => {
    setLoading(true)
    try {
      const data = await invoke<Server[]>('list_servers')
      setServers(data)
      setError(null)
    } catch (e) { setError(extractMessage(e)) }
    finally { setLoading(false) }
  }, [])

  useEffect(() => { fetch() }, [fetch])

  const deleteServer = async (id: string) => {
    await invoke('delete_server', { id })
    setServers(p => p.filter(s => s.id !== id))
  }

  return { servers, loading, error, refetch: fetch, deleteServer }
}

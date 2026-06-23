export interface User {
  id: string
  login: string
  email: string
  first_name: string | null
  last_name: string | null
  avatar_url: string | null
  subscription: 'free' | 'premium'
  created_at: string
}

export interface AuthResponse {
  token: string
  user: User
}

export interface Server {
  id: string
  name: string
  host: string
  ssh_user: string
  description: string | null
  is_active: boolean
  server_type: 'local' | 'remote'
  created_at: string
  updated_at: string
}

export interface Metric {
  id: number
  server_id: string
  collected_at: string
  cpu_usage: number
  cpu_cores: number | null
  ram_used: number
  ram_total: number
  ram_usage_pct: number
  temperature_cpu: number | null
  temperature_gpu: number | null
  disk_used: number | null
  disk_total: number | null
  disk_read_bytes_sec: number | null
  disk_write_bytes_sec: number | null
  net_rx_bytes_sec: number | null
  net_tx_bytes_sec: number | null
  load_avg_1: number | null
  load_avg_5: number | null
  load_avg_15: number | null
  uptime_seconds: number | null
}

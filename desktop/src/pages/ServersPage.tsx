import { useNavigate } from 'react-router-dom'
import { Server, Plus, Trash2, ChevronRight, Wifi, WifiOff, Monitor } from 'lucide-react'
import { PageHeader, Button, EmptyState, Spinner, Badge } from '@/components/ui'
import { useServers } from '@/hooks/useServers'
import type { Server as ServerType } from '@/types'
// @ts-ignore
import styles from './ServersPage.module.css'

export default function ServersPage() {
  const { servers, loading, deleteServer } = useServers()
  const navigate = useNavigate()

  if (loading) return <div className={styles.center}><Spinner size={28} /></div>

  return (
    <div>
      <PageHeader
        title="My Servers"
        subtitle={`${servers.length} server${servers.length !== 1 ? 's' : ''}`}
        action={
          <Button size="sm" onClick={() => navigate('/servers/add')}>
            <Plus size={14} /> Add server
          </Button>
        }
      />

      {servers.length === 0 ? (
        <EmptyState
          icon={<Server size={24} />}
          title="No servers yet"
          description="Add a local or remote Linux server to start monitoring."
          action={<Button onClick={() => navigate('/servers/add')}><Plus size={14} /> Add first server</Button>}
        />
      ) : (
        <div className={styles.list}>
          {servers.map(s => (
            <ServerRow key={s.id} server={s}
              onDelete={() => { if (confirm(`Delete "${s.name}"?`)) deleteServer(s.id) }}
            />
          ))}
        </div>
      )}
    </div>
  )
}

function ServerRow({ server: s, onDelete }: { server: ServerType; onDelete: () => void }) {
  const navigate = useNavigate()
  const isLocal = (s as any).server_type === 'local'

  return (
    <div className={styles.row} onClick={() => navigate(`/servers/${s.id}`)}>
      <div className={styles.rowIcon} style={{ borderColor: isLocal ? 'rgba(34,211,165,0.4)' : undefined }}>
        {isLocal ? <Monitor size={17} /> : <Server size={17} />}
      </div>
      <div className={styles.rowInfo}>
        <div className={styles.rowName}>
          {s.name}
          <span className={styles.rowId}>id: {s.id.slice(0, 8)}</span>
          <Badge color={isLocal ? 'green' : 'blue'}>{isLocal ? 'local' : 'remote'}</Badge>
        </div>
        <div className={styles.rowHost}>
          {isLocal
            ? <span className={styles.localTag}>localhost · /proc &amp; /sys</span>
            : <code>{s.ssh_user}@{s.host}</code>
          }
          {s.description && <span> · {s.description}</span>}
        </div>
      </div>
      <Badge color={s.is_active ? 'green' : 'red'}>
        {s.is_active
          ? <><Wifi size={9} style={{ marginRight: 3 }} />active</>
          : <><WifiOff size={9} style={{ marginRight: 3 }} />off</>
        }
      </Badge>
      <div className={styles.rowActions} onClick={e => e.stopPropagation()}>
        <button className={styles.delBtn} onClick={onDelete}><Trash2 size={14} /></button>
        <ChevronRight size={16} className={styles.chevron} />
      </div>
    </div>
  )
}

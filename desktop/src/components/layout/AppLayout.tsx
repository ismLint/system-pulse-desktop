import { Outlet, NavLink, useNavigate } from 'react-router-dom'
import { Activity, Server, User, LogOut, LayoutDashboard } from 'lucide-react'
import { useAuthStore } from '@/store/authStore'
import { invoke } from '@/utils/invoke'
import TitleBar from './TitleBar'
// @ts-ignore
import styles from './AppLayout.module.css'

const NAV = [
  { to: '/dashboard', icon: <LayoutDashboard size={16} />, label: 'Dashboard' },
  { to: '/servers',   icon: <Server size={16} />,          label: 'Servers' },
  { to: '/account',  icon: <User size={16} />,             label: 'Account' },
]

export default function AppLayout() {
  const { user, logout } = useAuthStore()
  const navigate = useNavigate()

  const handleLogout = async () => {
    await invoke('logout').catch(() => {})
    logout()
    navigate('/login')
  }

  return (
    <div className={styles.root}>
      <TitleBar />
      <div className={styles.body}>
        <aside className={styles.sidebar}>
          <div className={styles.logoWrap}>
            <Activity size={18} className={styles.logoIcon} />
            <span className={styles.logoText}>System<strong>Pulse</strong></span>
          </div>

          <nav className={styles.nav}>
            {NAV.map(item => (
              <NavLink
                key={item.to}
                to={item.to}
                className={({ isActive }) => `${styles.navItem} ${isActive ? styles.active : ''}`}
              >
                {item.icon}
                <span>{item.label}</span>
              </NavLink>
            ))}
          </nav>

          <div className={styles.sidebarBottom}>
            <div className={styles.userRow}>
              <div className={styles.avatar}>
                {user?.first_name?.[0] ?? user?.login?.[0]?.toUpperCase() ?? 'U'}
              </div>
              <div className={styles.userMeta}>
                <span className={styles.userName}>
                  {user?.first_name
                    ? `${user.first_name} ${user.last_name ?? ''}`.trim()
                    : user?.login}
                </span>
                <span className={styles.userSub}>{user?.subscription ?? 'free'}</span>
              </div>
              <button onClick={handleLogout} className={styles.logoutBtn} title="Sign out">
                <LogOut size={15} />
              </button>
            </div>
          </div>
        </aside>

        <main className={styles.content}>
          <Outlet />
        </main>
      </div>
    </div>
  )
}

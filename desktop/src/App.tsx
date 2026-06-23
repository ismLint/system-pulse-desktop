import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { useAuthStore } from '@/store/authStore'
import LoginPage from '@/pages/LoginPage'
import RegisterPage from '@/pages/RegisterPage'
import DashboardPage from '@/pages/DashboardPage'
import ServersPage from '@/pages/ServersPage'
import ServerDetailPage from '@/pages/ServerDetailPage'
import AddServerPage from '@/pages/AddServerPage'
import AccountPage from '@/pages/AccountPage'
import AppLayout from '@/components/layout/AppLayout'

function Private({ children }: { children: React.ReactNode }) {
  const { isAuthenticated } = useAuthStore()
  return isAuthenticated ? <>{children}</> : <Navigate to="/login" replace />
}

function PublicOnly({ children }: { children: React.ReactNode }) {
  const { isAuthenticated } = useAuthStore()
  return !isAuthenticated ? <>{children}</> : <Navigate to="/dashboard" replace />
}

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/login"    element={<PublicOnly><LoginPage /></PublicOnly>} />
        <Route path="/register" element={<PublicOnly><RegisterPage /></PublicOnly>} />

        <Route element={<Private><AppLayout /></Private>}>
          <Route path="/dashboard"   element={<DashboardPage />} />
          <Route path="/servers"     element={<ServersPage />} />
          <Route path="/servers/add" element={<AddServerPage />} />
          <Route path="/servers/:id" element={<ServerDetailPage />} />
          <Route path="/account"     element={<AccountPage />} />
        </Route>

        <Route path="*" element={<Navigate to="/login" replace />} />
      </Routes>
    </BrowserRouter>
  )
}

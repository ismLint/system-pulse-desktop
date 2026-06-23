import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import AuthForm from '@/components/auth/AuthForm'
import { invoke, extractMessage } from '@/utils/invoke'
import { useAuthStore } from '@/store/authStore'
import type { AuthResponse } from '@/types'

const FIELDS = [
  { name: 'login',    label: 'Login',    placeholder: 'your_login',   autoComplete: 'username' },
  { name: 'password', label: 'Password', type: 'password', placeholder: '••••••••', autoComplete: 'current-password' },
]

export default function LoginPage() {
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const { setAuth } = useAuthStore()
  const navigate = useNavigate()

  const handleSubmit = async (values: Record<string, string>) => {
    setLoading(true); setError(null)
    try {
      const res = await invoke<AuthResponse>('login', {
        input: { login: values.login, password: values.password },
      })
      setAuth(res.token, res.user)
      navigate('/dashboard')
    } catch (e) { setError(extractMessage(e)) }
    finally { setLoading(false) }
  }

  return (
    <AuthForm
      title="Welcome back"
      subtitle="Sign in to System Pulse"
      fields={FIELDS}
      submitLabel="Sign in"
      loading={loading}
      error={error}
      onSubmit={handleSubmit}
      footer={<>No account? <Link to="/register">Create one</Link></>}
    />
  )
}

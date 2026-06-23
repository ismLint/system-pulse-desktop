import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import AuthForm from '@/components/auth/AuthForm'
import { invoke, extractMessage } from '@/utils/invoke'
import { useAuthStore } from '@/store/authStore'
import type { AuthResponse } from '@/types'

const FIELDS = [
  { name: 'login',      label: 'Login',      placeholder: 'your_login',       autoComplete: 'username' },
  { name: 'email',      label: 'E-mail',     type: 'email', placeholder: 'you@example.com', autoComplete: 'email' },
  { name: 'password',   label: 'Password',   type: 'password', placeholder: '8+ characters', autoComplete: 'new-password' },
  { name: 'first_name', label: 'First name', placeholder: 'John' },
  { name: 'last_name',  label: 'Last name',  placeholder: 'Doe' },
]

export default function RegisterPage() {
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const { setAuth } = useAuthStore()
  const navigate = useNavigate()

  const handleSubmit = async (values: Record<string, string>) => {
    setLoading(true); setError(null)
    try {
      const res = await invoke<AuthResponse>('register', {
        input: {
          login: values.login, email: values.email, password: values.password,
          first_name: values.first_name || null, last_name: values.last_name || null,
        },
      })
      setAuth(res.token, res.user)
      navigate('/dashboard')
    } catch (e) { setError(extractMessage(e)) }
    finally { setLoading(false) }
  }

  return (
    <AuthForm
      title="Create account"
      subtitle="Start monitoring your servers"
      fields={FIELDS}
      submitLabel="Create account"
      loading={loading}
      error={error}
      onSubmit={handleSubmit}
      footer={<>Already have an account? <Link to="/login">Sign in</Link></>}
    />
  )
}

import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { useAuthStore } from '@/store/authStore'

export async function invoke<T>(
  cmd: string,
  args: Record<string, unknown> = {}
): Promise<T> {
  const token = useAuthStore.getState().token
  if (token && !args.token) {
    args = { ...args, token }
  }
  try {
    return await tauriInvoke<T>(cmd, args)
  } catch (err) {
    // Tauri returns serialized AppError or string
    if (typeof err === 'object' && err !== null && 'message' in err) {
      throw new Error((err as { message: string }).message)
    }
    throw new Error(String(err))
  }
}

export function extractMessage(err: unknown): string {
  if (err instanceof Error) return err.message
  if (typeof err === 'string') return err
  return 'Unknown error'
}

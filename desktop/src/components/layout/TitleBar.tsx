import { Minus, Square, X, Activity } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
// @ts-ignore
import styles from './TitleBar.module.css'

export default function TitleBar() {
  return (
    <div className={styles.bar} data-tauri-drag-region>
      <div className={styles.left} data-tauri-drag-region>
        <Activity size={14} className={styles.icon} />
        <span className={styles.title} data-tauri-drag-region>System Pulse</span>
      </div>
      <div className={styles.controls}>
        <button onClick={() => invoke('minimize_window')} className={styles.btn} title="Minimize">
          <Minus size={12} />
        </button>
        <button onClick={() => invoke('maximize_window')} className={styles.btn} title="Maximize">
          <Square size={11} />
        </button>
        <button onClick={() => invoke('close_window')} className={`${styles.btn} ${styles.close}`} title="Close">
          <X size={12} />
        </button>
      </div>
    </div>
  )
}

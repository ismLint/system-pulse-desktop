import {
  AreaChart, Area, XAxis, YAxis, CartesianGrid,
  Tooltip, ResponsiveContainer, ReferenceLine,
} from 'recharts'
// @ts-ignore
import styles from './MetricChart.module.css'

interface Props {
  title: string
  data: Record<string, string | number>[]
  dataKey?: string
  color?: string
  unit?: string
  domain?: [number | string, number | string]
  height?: number
  referenceValue?: number
  formatValue?: (v: number) => string
}

export default function MetricChart({
  title, data, dataKey = 'value', color = 'var(--chart-cpu)',
  unit = '%', domain = [0, 100], height = 180,
  referenceValue, formatValue = (v) => v.toFixed(1),
}: Props) {
  const latest = data.length > 0
    ? (data[data.length - 1][dataKey] as number)
    : null

  const Tip = ({ active, payload, label }: any) => {
    if (!active || !payload?.length) return null
    return (
      <div className={styles.tip}>
        <p className={styles.tipTime}>{label}</p>
        <p className={styles.tipVal} style={{ color }}>
          {formatValue(payload[0].value)}{unit}
        </p>
      </div>
    )
  }

  return (
    <div className={styles.wrap}>
      <div className={styles.head}>
        <span className={styles.title}>{title}</span>
        {latest !== null && (
          <span className={styles.current} style={{ color }}>
            {formatValue(latest)}{unit}
          </span>
        )}
      </div>
      <ResponsiveContainer width="100%" height={height}>
        <AreaChart data={data} margin={{ top: 4, right: 2, left: -22, bottom: 0 }}>
          <defs>
            <linearGradient id={`g-${title}`} x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%"  stopColor={color} stopOpacity={0.22} />
              <stop offset="95%" stopColor={color} stopOpacity={0.02} />
            </linearGradient>
          </defs>
          <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.04)" vertical={false} />
          <XAxis dataKey="time"
            tick={{ fill: 'var(--text-muted)', fontSize: 9, fontFamily: 'var(--font-mono)' }}
            tickLine={false} axisLine={false} interval="preserveStartEnd" />
          <YAxis domain={domain}
            tick={{ fill: 'var(--text-muted)', fontSize: 9, fontFamily: 'var(--font-mono)' }}
            tickLine={false} axisLine={false} />
          <Tooltip content={<Tip />} />
          {referenceValue !== undefined && (
            <ReferenceLine y={referenceValue} stroke="var(--warning)"
              strokeDasharray="4 4" strokeWidth={1} />
          )}
          <Area type="monotone" dataKey={dataKey} stroke={color} strokeWidth={1.5}
            fill={`url(#g-${title})`} dot={false}
            activeDot={{ r: 3, fill: color, stroke: 'var(--bg-base)', strokeWidth: 2 }}
            isAnimationActive={false} />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  )
}

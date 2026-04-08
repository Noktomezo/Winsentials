import { useId } from 'react'
import { Area, AreaChart, ResponsiveContainer, Tooltip, YAxis } from 'recharts'

export interface ChartPoint {
  value: number
}

interface LiveChartProps {
  data: ChartPoint[]
  yDomain?: [number | 'auto' | 'dataMin' | 'dataMax', number | 'auto' | 'dataMin' | 'dataMax']
  unit?: string
  height?: number
}

export function LiveChart({ data, yDomain, unit = '', height = 80 }: LiveChartProps) {
  const gradientId = useId().replace(/:/g, '-')

  return (
    <ResponsiveContainer height={height} width="100%">
      <AreaChart data={data} margin={{ top: 2, right: 0, bottom: 0, left: 0 }}>
        <defs>
          <linearGradient id={gradientId} x1="0" x2="0" y1="0" y2="1">
            <stop offset="5%" stopColor="var(--metric-accent)" stopOpacity={0.3} />
            <stop offset="95%" stopColor="var(--metric-accent)" stopOpacity={0.03} />
          </linearGradient>
        </defs>
        <YAxis domain={yDomain ?? ['auto', 'auto']} hide />
        <Tooltip
          contentStyle={{
            background: 'var(--card)',
            border: '1px solid var(--border)',
            borderRadius: '6px',
            fontSize: '11px',
            padding: '4px 8px',
          }}
          formatter={value => [typeof value === 'number' ? `${value.toFixed(1)}${unit}` : '', '']}
          itemStyle={{ color: 'var(--foreground)' }}
          labelFormatter={() => ''}
          separator=""
        />
        <Area
          dataKey="value"
          dot={false}
          fill={`url(#${gradientId})`}
          isAnimationActive={false}
          stroke="var(--metric-accent)"
          strokeWidth={1.5}
          type="monotone"
        />
      </AreaChart>
    </ResponsiveContainer>
  )
}

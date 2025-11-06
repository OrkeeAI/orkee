// ABOUTME: Resource monitoring component for sandbox metrics visualization
// ABOUTME: Displays CPU, memory, disk, and network usage with real-time graphs

import { useState, useEffect, useCallback } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { getSandboxMetrics, type ResourceMetrics } from '@/services/sandbox'
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from 'recharts'
import { Cpu, HardDrive, Activity, Network, RefreshCw } from 'lucide-react'
import { Button } from '@/components/ui/button'

interface ResourceMonitorProps {
  sandboxId: string
}

interface MetricsHistory {
  timestamp: string
  cpuPercent: number
  memoryUsedMb: number
  memoryPercentage: number
  diskUsedGb: number
  diskPercentage: number
  networkRxMb: number
  networkTxMb: number
}

export function ResourceMonitor({ sandboxId }: ResourceMonitorProps) {
  const [currentMetrics, setCurrentMetrics] = useState<ResourceMetrics | null>(null)
  const [metricsHistory, setMetricsHistory] = useState<MetricsHistory[]>([])
  const [loading, setLoading] = useState(false)

  const loadMetrics = useCallback(async () => {
    try {
      const metrics = await getSandboxMetrics(sandboxId)
      setCurrentMetrics(metrics)

      // Add to history (keep last 20 data points)
      setMetricsHistory((prev) => {
        const newHistory = [
          ...prev,
          {
            timestamp: new Date(metrics.timestamp).toLocaleTimeString(),
            cpuPercent: metrics.cpu_usage_percent,
            memoryUsedMb: metrics.memory_usage_mb,
            memoryPercentage: (metrics.memory_usage_mb / metrics.memory_limit_mb) * 100,
            diskUsedGb: metrics.disk_usage_gb,
            diskPercentage: (metrics.disk_usage_gb / metrics.disk_limit_gb) * 100,
            networkRxMb: metrics.network_rx_bytes / (1024 * 1024),
            networkTxMb: metrics.network_tx_bytes / (1024 * 1024),
          },
        ]
        return newHistory.slice(-20)
      })
    } catch (error) {
      console.error('Failed to load metrics:', error)
    }
  }, [sandboxId])

  useEffect(() => {
    loadMetrics()
    const interval = setInterval(loadMetrics, 5000) // Update every 5 seconds

    return () => clearInterval(interval)
  }, [loadMetrics])

  const handleRefresh = async () => {
    setLoading(true)
    await loadMetrics()
    setLoading(false)
  }

  const formatBytes = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
  }

  if (!currentMetrics) {
    return (
      <div className="flex items-center justify-center h-full text-muted-foreground">
        <RefreshCw className="h-5 w-5 animate-spin mr-2" />
        Loading metrics...
      </div>
    )
  }

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold">Resource Monitor</h3>
        <Button variant="outline" size="sm" onClick={handleRefresh} disabled={loading}>
          <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
        </Button>
      </div>

      {/* Current Metrics Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">CPU Usage</CardTitle>
            <Cpu className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{currentMetrics.cpu_usage_percent.toFixed(1)}%</div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Memory</CardTitle>
            <Activity className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {currentMetrics.memory_usage_mb.toFixed(0)} MB
            </div>
            <p className="text-xs text-muted-foreground">
              of {currentMetrics.memory_limit_mb.toFixed(0)} MB (
              {((currentMetrics.memory_usage_mb / currentMetrics.memory_limit_mb) * 100).toFixed(1)}%)
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Disk</CardTitle>
            <HardDrive className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {currentMetrics.disk_usage_gb.toFixed(1)} GB
            </div>
            <p className="text-xs text-muted-foreground">
              of {currentMetrics.disk_limit_gb.toFixed(0)} GB (
              {((currentMetrics.disk_usage_gb / currentMetrics.disk_limit_gb) * 100).toFixed(1)}%)
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Network</CardTitle>
            <Network className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-sm font-bold">
              ↓ {formatBytes(currentMetrics.network_rx_bytes)}
            </div>
            <div className="text-sm font-bold">
              ↑ {formatBytes(currentMetrics.network_tx_bytes)}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Charts */}
      <div className="space-y-4">
        {/* CPU Chart */}
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">CPU Usage Over Time</CardTitle>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={200}>
              <LineChart data={metricsHistory}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="timestamp" fontSize={12} />
                <YAxis fontSize={12} domain={[0, 100]} />
                <Tooltip />
                <Legend />
                <Line type="monotone" dataKey="cpuPercent" stroke="#8884d8" name="CPU %" />
              </LineChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        {/* Memory Chart */}
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Memory Usage Over Time</CardTitle>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={200}>
              <LineChart data={metricsHistory}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="timestamp" fontSize={12} />
                <YAxis fontSize={12} />
                <Tooltip />
                <Legend />
                <Line type="monotone" dataKey="memoryUsedMb" stroke="#82ca9d" name="Memory (MB)" />
              </LineChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        {/* Network Chart */}
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Network Activity</CardTitle>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={200}>
              <LineChart data={metricsHistory}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="timestamp" fontSize={12} />
                <YAxis fontSize={12} />
                <Tooltip />
                <Legend />
                <Line type="monotone" dataKey="networkRxMb" stroke="#ffc658" name="RX (MB)" />
                <Line type="monotone" dataKey="networkTxMb" stroke="#ff7300" name="TX (MB)" />
              </LineChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}

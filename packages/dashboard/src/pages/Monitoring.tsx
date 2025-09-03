import { Monitor, Activity, AlertCircle, CheckCircle, Clock } from 'lucide-react'

interface SystemMetric {
  name: string
  value: string
  status: 'healthy' | 'warning' | 'error'
  trend: 'up' | 'down' | 'stable'
}

interface AgentStatus {
  id: number
  name: string
  status: 'running' | 'stopped' | 'error'
  uptime: string
  requests: number
  lastActivity: Date
}

export function Monitoring() {
  const systemMetrics: SystemMetric[] = [
    { name: 'CPU Usage', value: '45%', status: 'healthy', trend: 'stable' },
    { name: 'Memory Usage', value: '72%', status: 'warning', trend: 'up' },
    { name: 'Network I/O', value: '1.2 GB/s', status: 'healthy', trend: 'up' },
    { name: 'Response Time', value: '245ms', status: 'healthy', trend: 'down' },
  ]

  const agents: AgentStatus[] = [
    {
      id: 1,
      name: 'Customer Support Agent',
      status: 'running',
      uptime: '2d 14h 32m',
      requests: 1247,
      lastActivity: new Date()
    },
    {
      id: 2,
      name: 'Data Processing Agent',
      status: 'running', 
      uptime: '1d 8h 15m',
      requests: 892,
      lastActivity: new Date(Date.now() - 120000) // 2 minutes ago
    },
    {
      id: 3,
      name: 'Content Moderator',
      status: 'error',
      uptime: '0d 0h 0m',
      requests: 0,
      lastActivity: new Date(Date.now() - 900000) // 15 minutes ago
    },
    {
      id: 4,
      name: 'Analytics Agent',
      status: 'stopped',
      uptime: '0d 0h 0m', 
      requests: 45,
      lastActivity: new Date(Date.now() - 3600000) // 1 hour ago
    }
  ]

  const getStatusIcon = (status: AgentStatus['status']) => {
    switch (status) {
      case 'running': return <CheckCircle className="h-4 w-4 text-green-500" />
      case 'stopped': return <Clock className="h-4 w-4 text-gray-500" />
      case 'error': return <AlertCircle className="h-4 w-4 text-red-500" />
    }
  }

  const getMetricColor = (status: SystemMetric['status']) => {
    switch (status) {
      case 'healthy': return 'text-green-600'
      case 'warning': return 'text-yellow-600'
      case 'error': return 'text-red-600'
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Monitoring</h1>
        <p className="text-muted-foreground">
          Monitor system health, agent performance, and resource usage.
        </p>
      </div>

      {/* System Metrics */}
      <div>
        <h2 className="text-xl font-semibold mb-4">System Metrics</h2>
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          {systemMetrics.map((metric, index) => (
            <div key={index} className="rounded-lg border p-6">
              <div className="flex items-center gap-2 mb-2">
                <Activity className="h-4 w-4 text-primary" />
                <h3 className="text-sm font-medium">{metric.name}</h3>
              </div>
              <p className={`text-2xl font-bold ${getMetricColor(metric.status)}`}>
                {metric.value}
              </p>
              <p className="text-xs text-muted-foreground capitalize">
                {metric.trend} trend • {metric.status}
              </p>
            </div>
          ))}
        </div>
      </div>

      {/* Agent Status */}
      <div>
        <h2 className="text-xl font-semibold mb-4">Agent Status</h2>
        <div className="rounded-lg border">
          <div className="p-4">
            <div className="grid gap-4">
              {agents.map((agent) => (
                <div key={agent.id} className="flex items-center justify-between p-4 rounded-lg border bg-card">
                  <div className="flex items-center gap-3">
                    {getStatusIcon(agent.status)}
                    <div>
                      <h3 className="font-medium">{agent.name}</h3>
                      <p className="text-sm text-muted-foreground">
                        {agent.status === 'running' ? `Uptime: ${agent.uptime}` : 'Not running'}
                      </p>
                    </div>
                  </div>
                  <div className="text-right text-sm">
                    <p className="font-medium">{agent.requests} requests</p>
                    <p className="text-muted-foreground">
                      Last: {agent.lastActivity.toLocaleTimeString()}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* Real-time Chart Placeholder */}
      <div>
        <h2 className="text-xl font-semibold mb-4">Performance Charts</h2>
        <div className="rounded-lg border p-6">
          <div className="h-64 bg-muted rounded-md flex items-center justify-center">
            <div className="text-center">
              <Monitor className="h-8 w-8 text-muted-foreground mx-auto mb-2" />
              <p className="text-muted-foreground">Real-time performance charts will be displayed here</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
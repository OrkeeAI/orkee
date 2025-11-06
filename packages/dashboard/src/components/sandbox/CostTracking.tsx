// ABOUTME: Cost tracking dashboard for sandbox usage costs
// ABOUTME: Displays cost breakdown by sandbox and provider with alerts

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
import { type Sandbox } from '@/services/sandbox'
import { DollarSign, TrendingUp, AlertTriangle, Server } from 'lucide-react'
import { Alert, AlertDescription } from '@/components/ui/alert'

interface CostTrackingProps {
  sandboxes: Sandbox[]
  maxTotalCost: number
  costAlertThreshold: number
}

export function CostTracking({ sandboxes, maxTotalCost, costAlertThreshold }: CostTrackingProps) {
  const totalCost = sandboxes.reduce((sum, sandbox) => sum + (sandbox.total_cost || 0), 0)
  const costPercentage = (totalCost / maxTotalCost) * 100

  const costByProvider = sandboxes.reduce((acc, sandbox) => {
    acc[sandbox.provider] = (acc[sandbox.provider] || 0) + (sandbox.total_cost || 0)
    return acc
  }, {} as Record<string, number>)

  const topSandboxesByCost = [...sandboxes]
    .sort((a, b) => (b.total_cost || 0) - (a.total_cost || 0))
    .slice(0, 5)

  const formatCost = (cost: number | undefined | null): string => {
    if (cost === undefined || cost === null) return '$0.00'
    return `$${cost.toFixed(2)}`
  }

  const isOverThreshold = totalCost >= costAlertThreshold

  return (
    <div className="space-y-4">
      {/* Total Cost Card */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <DollarSign className="h-5 w-5" />
            Total Cost
          </CardTitle>
          <CardDescription>
            Current spend across all sandboxes
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <div className="flex items-end justify-between mb-2">
              <div className="text-3xl font-bold">{formatCost(totalCost)}</div>
              <div className="text-sm text-muted-foreground">
                of {formatCost(maxTotalCost)} limit
              </div>
            </div>
            <Progress value={costPercentage} className="h-2" />
            <div className="flex items-center justify-between mt-2">
              <span className="text-sm text-muted-foreground">
                {costPercentage.toFixed(1)}% of budget used
              </span>
              {isOverThreshold && (
                <Badge variant="destructive">Over threshold</Badge>
              )}
            </div>
          </div>

          {isOverThreshold && (
            <Alert variant="destructive">
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription>
                Cost has exceeded the alert threshold of {formatCost(costAlertThreshold)}.
                Consider stopping unused sandboxes to reduce costs.
              </AlertDescription>
            </Alert>
          )}
        </CardContent>
      </Card>

      {/* Cost by Provider */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Server className="h-5 w-5" />
            Cost by Provider
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {Object.entries(costByProvider).map(([provider, cost]) => {
              const percentage = (cost / totalCost) * 100
              return (
                <div key={provider}>
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-sm font-medium capitalize">{provider}</span>
                    <span className="text-sm font-medium">{formatCost(cost)}</span>
                  </div>
                  <Progress value={percentage} className="h-2" />
                  <div className="text-xs text-muted-foreground mt-1">
                    {percentage.toFixed(1)}% of total
                  </div>
                </div>
              )
            })}
            {Object.keys(costByProvider).length === 0 && (
              <p className="text-sm text-muted-foreground text-center py-4">
                No active sandboxes
              </p>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Top Sandboxes by Cost */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <TrendingUp className="h-5 w-5" />
            Top Sandboxes by Cost
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            {topSandboxesByCost.map((sandbox, index) => (
              <div
                key={sandbox.id}
                className="flex items-center justify-between p-2 rounded hover:bg-accent"
              >
                <div className="flex items-center gap-2">
                  <Badge variant="outline" className="w-6 h-6 flex items-center justify-center p-0">
                    {index + 1}
                  </Badge>
                  <div>
                    <p className="text-sm font-medium">{sandbox.name}</p>
                    <p className="text-xs text-muted-foreground capitalize">
                      {sandbox.provider} • {sandbox.status}
                    </p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="text-sm font-bold">{formatCost(sandbox.total_cost)}</p>
                  <p className="text-xs text-muted-foreground">
                    {formatCost(sandbox.cost_per_hour)}/hr
                  </p>
                </div>
              </div>
            ))}
            {topSandboxesByCost.length === 0 && (
              <p className="text-sm text-muted-foreground text-center py-4">
                No active sandboxes
              </p>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

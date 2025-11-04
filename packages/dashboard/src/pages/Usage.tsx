// ABOUTME: AI Usage dashboard page showing token usage, costs, and tool statistics
// ABOUTME: Displays key metrics and visualizations for monitoring AI API consumption

import { useEffect, useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { BarChart, Bar, LineChart, Line, PieChart, Pie, Cell, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from 'recharts';
import { Activity, DollarSign, MessageSquare, Wrench, AlertCircle, BarChart3 } from 'lucide-react';
import {
  getAiUsageStats,
  getToolUsageStats,
  getTimeSeriesData,
  formatCost,
  formatTokens,
  formatDuration,
  type AiUsageStats,
  type ToolUsageStats,
  type TimeSeriesDataPoint,
} from '@/services/aiUsage';

const CHART_COLORS = [
  'hsl(var(--chart-1))',
  'hsl(var(--chart-2))',
  'hsl(var(--chart-3))',
  'hsl(var(--chart-4))',
  'hsl(var(--chart-5))',
];

export default function Usage() {
  const [stats, setStats] = useState<AiUsageStats | null>(null);
  const [toolStats, setToolStats] = useState<ToolUsageStats[]>([]);
  const [timeSeriesData, setTimeSeriesData] = useState<TimeSeriesDataPoint[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function loadData() {
      try {
        setLoading(true);
        setError(null);

        const [statsResponse, toolStatsResponse, timeSeriesResponse] = await Promise.all([
          getAiUsageStats(),
          getToolUsageStats(),
          getTimeSeriesData({ interval: 'day' }),
        ]);

        if (statsResponse.data && statsResponse.data.data) {
          // Transform snake_case to camelCase
          const rawStats = statsResponse.data.data;
          setStats({
            totalRequests: rawStats.total_requests,
            successfulRequests: rawStats.successful_requests,
            failedRequests: rawStats.failed_requests,
            totalInputTokens: rawStats.total_input_tokens,
            totalOutputTokens: rawStats.total_output_tokens,
            totalTokens: rawStats.total_tokens,
            totalCost: rawStats.total_cost,
            averageDurationMs: rawStats.average_duration_ms,
            byOperation: rawStats.byOperation || [],
            byModel: rawStats.byModel || [],
            byProvider: rawStats.byProvider || [],
          });
        }

        if (toolStatsResponse.data && toolStatsResponse.data.data) {
          setToolStats(toolStatsResponse.data.data);
        }

        if (timeSeriesResponse.data && timeSeriesResponse.data.data) {
          setTimeSeriesData(timeSeriesResponse.data.data);
        }
      } catch (err) {
        console.error('Failed to load usage data:', err);
        setError(err instanceof Error ? err.message : 'Failed to load usage data');
      } finally {
        setLoading(false);
      }
    }

    loadData();
  }, []);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  if (error) {
    return (
      <Alert variant="destructive">
        <AlertCircle className="h-4 w-4" />
        <AlertDescription>{error}</AlertDescription>
      </Alert>
    );
  }

  if (!stats) {
    return (
      <Alert>
        <AlertCircle className="h-4 w-4" />
        <AlertDescription>No usage data available</AlertDescription>
      </Alert>
    );
  }

  const successRate = stats.totalRequests > 0
    ? ((stats.successfulRequests / stats.totalRequests) * 100).toFixed(1)
    : '0';

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">AI Usage</h1>
        <p className="text-muted-foreground mt-2">
          Monitor your AI API usage, costs, and tool performance
        </p>
      </div>

      <Tabs defaultValue="overview" className="space-y-6">
        <TabsList>
          <TabsTrigger value="overview">
            <Activity className="h-4 w-4 mr-2" />
            Overview
          </TabsTrigger>
          <TabsTrigger value="charts">
            <BarChart3 className="h-4 w-4 mr-2" />
            Charts & Analytics
          </TabsTrigger>
        </TabsList>

        {/* Overview Tab - Key Metrics and Summaries */}
        <TabsContent value="overview" className="space-y-6">
          {/* Key Metrics Cards */}
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">Total Requests</CardTitle>
                <MessageSquare className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{stats.totalRequests.toLocaleString()}</div>
                <p className="text-xs text-muted-foreground mt-1">
                  {successRate}% success rate
                </p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">Total Tokens</CardTitle>
                <Activity className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{formatTokens(stats.totalTokens)}</div>
                <p className="text-xs text-muted-foreground mt-1">
                  {formatTokens(stats.totalInputTokens)} in / {formatTokens(stats.totalOutputTokens)} out
                </p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">Total Cost</CardTitle>
                <DollarSign className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{formatCost(stats.totalCost)}</div>
                <p className="text-xs text-muted-foreground mt-1">
                  Avg {formatDuration(stats.averageDurationMs)}/request
                </p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">Tool Calls</CardTitle>
                <Wrench className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {toolStats.reduce((sum, tool) => sum + tool.call_count, 0).toLocaleString()}
                </div>
                <p className="text-xs text-muted-foreground mt-1">
                  {toolStats.length} unique tools
                </p>
              </CardContent>
            </Card>
          </div>

          {/* Breakdown by Model and Provider */}
          <div className="grid gap-4 md:grid-cols-2">
            <Card>
              <CardHeader>
                <CardTitle>By Model</CardTitle>
                <CardDescription>Usage breakdown by AI model</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {stats.byModel.slice(0, 5).map((model) => (
                    <div key={model.model} className="flex justify-between items-center">
                      <span className="text-sm font-medium">{model.model}</span>
                      <div className="text-sm text-muted-foreground">
                        {formatTokens(model.totalTokens)} • {formatCost(model.totalCost)}
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>By Provider</CardTitle>
                <CardDescription>Usage breakdown by provider</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {stats.byProvider.slice(0, 5).map((provider) => (
                    <div key={provider.provider} className="flex justify-between items-center">
                      <span className="text-sm font-medium">{provider.provider}</span>
                      <div className="text-sm text-muted-foreground">
                        {formatTokens(provider.totalTokens)} • {formatCost(provider.totalCost)}
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </div>

          {/* Top Tools Summary */}
          {toolStats.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle>Top Tools</CardTitle>
                <CardDescription>Most frequently used tools</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {toolStats.slice(0, 5).map((tool) => {
                    const successRate = tool.call_count > 0
                      ? ((tool.success_count / tool.call_count) * 100).toFixed(1)
                      : '0';
                    return (
                      <div key={tool.tool_name} className="flex justify-between items-center">
                        <span className="text-sm font-medium">{tool.tool_name}</span>
                        <div className="text-sm text-muted-foreground">
                          {tool.call_count} calls • {successRate}% success • {formatDuration(tool.average_duration_ms)}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </CardContent>
            </Card>
          )}
        </TabsContent>

        {/* Charts Tab - Advanced Visualizations */}
        <TabsContent value="charts" className="space-y-6">
          {/* Usage Trend Line Chart */}
          {timeSeriesData.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle>Usage Trend Over Time</CardTitle>
                <CardDescription>Daily requests, tokens, and costs</CardDescription>
              </CardHeader>
              <CardContent>
                <ResponsiveContainer width="100%" height={350}>
                  <LineChart data={timeSeriesData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis
                      dataKey="timestamp"
                      tickFormatter={(value) => new Date(value).toLocaleDateString()}
                    />
                    <YAxis yAxisId="left" />
                    <YAxis yAxisId="right" orientation="right" />
                    <Tooltip
                      labelFormatter={(value) => new Date(value as string).toLocaleDateString()}
                      formatter={(value: number, name: string) => {
                        if (name === 'cost') return [formatCost(value), 'Cost'];
                        if (name === 'token_count') return [formatTokens(value), 'Tokens'];
                        if (name === 'request_count') return [value.toLocaleString(), 'Requests'];
                        if (name === 'tool_call_count') return [value.toLocaleString(), 'Tool Calls'];
                        return [value, name];
                      }}
                    />
                    <Legend />
                    <Line
                      yAxisId="left"
                      type="monotone"
                      dataKey="request_count"
                      stroke={CHART_COLORS[0]}
                      name="Requests"
                    />
                    <Line
                      yAxisId="left"
                      type="monotone"
                      dataKey="token_count"
                      stroke={CHART_COLORS[1]}
                      name="Tokens"
                    />
                    <Line
                      yAxisId="right"
                      type="monotone"
                      dataKey="cost"
                      stroke={CHART_COLORS[2]}
                      name="Cost"
                    />
                  </LineChart>
                </ResponsiveContainer>
              </CardContent>
            </Card>
          )}

          {/* Tool Usage Bar Charts */}
          {toolStats.length > 0 && (
            <div className="grid gap-4 md:grid-cols-2">
              <Card>
                <CardHeader>
                  <CardTitle>Tool Call Statistics</CardTitle>
                  <CardDescription>Call counts and success rates</CardDescription>
                </CardHeader>
                <CardContent>
                  <ResponsiveContainer width="100%" height={300}>
                    <BarChart data={toolStats.slice(0, 10)}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis dataKey="tool_name" angle={-45} textAnchor="end" height={100} />
                      <YAxis />
                      <Tooltip
                        formatter={(value: number, name: string) => {
                          if (name === 'average_duration_ms') return [formatDuration(value), 'Avg Duration'];
                          return [value.toLocaleString(), name];
                        }}
                      />
                      <Legend />
                      <Bar dataKey="call_count" fill={CHART_COLORS[0]} name="Total Calls" />
                      <Bar dataKey="success_count" fill={CHART_COLORS[1]} name="Successful" />
                      <Bar dataKey="failure_count" fill={CHART_COLORS[2]} name="Failed" />
                    </BarChart>
                  </ResponsiveContainer>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>Tool Performance</CardTitle>
                  <CardDescription>Average execution time per tool</CardDescription>
                </CardHeader>
                <CardContent>
                  <ResponsiveContainer width="100%" height={300}>
                    <BarChart data={toolStats.slice(0, 10)}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis dataKey="tool_name" angle={-45} textAnchor="end" height={100} />
                      <YAxis />
                      <Tooltip
                        formatter={(value: number) => [formatDuration(value), 'Avg Duration']}
                      />
                      <Legend />
                      <Bar dataKey="average_duration_ms" fill={CHART_COLORS[3]} name="Avg Duration (ms)" />
                    </BarChart>
                  </ResponsiveContainer>
                </CardContent>
              </Card>
            </div>
          )}

          {/* Model and Provider Breakdown Pie Charts */}
          <div className="grid gap-4 md:grid-cols-2">
            {stats.byModel.length > 0 && (
              <Card>
                <CardHeader>
                  <CardTitle>Models by Token Usage</CardTitle>
                  <CardDescription>Token distribution across models</CardDescription>
                </CardHeader>
                <CardContent>
                  <ResponsiveContainer width="100%" height={300}>
                    <PieChart>
                      <Pie
                        data={stats.byModel.slice(0, 5)}
                        dataKey="totalTokens"
                        nameKey="model"
                        cx="50%"
                        cy="50%"
                        outerRadius={80}
                        label={(entry) => `${entry.model}: ${formatTokens(entry.totalTokens)}`}
                      >
                        {stats.byModel.slice(0, 5).map((_, index) => (
                          <Cell key={`cell-${index}`} fill={CHART_COLORS[index % CHART_COLORS.length]} />
                        ))}
                      </Pie>
                      <Tooltip formatter={(value: number) => formatTokens(value)} />
                    </PieChart>
                  </ResponsiveContainer>
                </CardContent>
              </Card>
            )}

            {stats.byProvider.length > 0 && (
              <Card>
                <CardHeader>
                  <CardTitle>Providers by Cost</CardTitle>
                  <CardDescription>Cost distribution across providers</CardDescription>
                </CardHeader>
                <CardContent>
                  <ResponsiveContainer width="100%" height={300}>
                    <PieChart>
                      <Pie
                        data={stats.byProvider.slice(0, 5)}
                        dataKey="totalCost"
                        nameKey="provider"
                        cx="50%"
                        cy="50%"
                        outerRadius={80}
                        label={(entry) => `${entry.provider}: ${formatCost(entry.totalCost)}`}
                      >
                        {stats.byProvider.slice(0, 5).map((_, index) => (
                          <Cell key={`cell-${index}`} fill={CHART_COLORS[index % CHART_COLORS.length]} />
                        ))}
                      </Pie>
                      <Tooltip formatter={(value: number) => formatCost(value)} />
                    </PieChart>
                  </ResponsiveContainer>
                </CardContent>
              </Card>
            )}
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}

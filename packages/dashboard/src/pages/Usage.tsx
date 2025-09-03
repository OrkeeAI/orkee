export function Usage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Usage</h1>
        <p className="text-muted-foreground">
          Monitor your API usage, token consumption, and costs.
        </p>
      </div>
      
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <div className="rounded-lg border p-6">
          <h3 className="text-sm font-medium">Total Requests</h3>
          <p className="text-2xl font-bold">1,234</p>
        </div>
        <div className="rounded-lg border p-6">
          <h3 className="text-sm font-medium">Tokens Used</h3>
          <p className="text-2xl font-bold">456.7K</p>
        </div>
        <div className="rounded-lg border p-6">
          <h3 className="text-sm font-medium">Active Agents</h3>
          <p className="text-2xl font-bold">12</p>
        </div>
        <div className="rounded-lg border p-6">
          <h3 className="text-sm font-medium">Monthly Cost</h3>
          <p className="text-2xl font-bold">$89.40</p>
        </div>
      </div>

      <div className="rounded-lg border p-6">
        <h3 className="text-lg font-medium mb-4">Usage Graph</h3>
        <div className="h-64 bg-muted rounded-md flex items-center justify-center">
          <p className="text-muted-foreground">Usage chart will be displayed here</p>
        </div>
      </div>
    </div>
  )
}
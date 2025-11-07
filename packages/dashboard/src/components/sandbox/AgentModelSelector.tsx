// ABOUTME: Agent and model selector component for sandbox configuration
// ABOUTME: Allows selection of AI agent and specific model for sandbox execution

import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

// Agent configuration (from sandboxes.md)
const AGENTS = [
  {
    id: 'claude',
    name: 'Claude',
    models: [
      { id: 'claude-sonnet-4-5-20250929', name: 'Claude Sonnet 4.5' },
      { id: 'claude-haiku-4-5-20251001', name: 'Claude Haiku 4.5' },
      { id: 'claude-opus-4-1-20250805', name: 'Claude Opus 4' },
    ],
  },
  {
    id: 'codex',
    name: 'Codex',
    models: [
      { id: 'gpt-5-codex', name: 'GPT-5 Codex' },
      { id: 'gpt-5', name: 'GPT-5' },
      { id: 'gpt-5-mini', name: 'GPT-5 Mini' },
      { id: 'gpt-4o', name: 'GPT-4o' },
    ],
  },
  {
    id: 'gemini',
    name: 'Gemini',
    models: [
      { id: 'gemini-2.5-pro', name: 'Gemini 2.5 Pro' },
      { id: 'gemini-2.5-flash', name: 'Gemini 2.5 Flash' },
      { id: 'gemini-2.5-flash-lite', name: 'Gemini 2.5 Flash Lite' },
    ],
  },
  {
    id: 'grok',
    name: 'Grok',
    models: [
      { id: 'grok-4-latest', name: 'Grok 4 Latest' },
      { id: 'grok-4-fast', name: 'Grok 4 Fast' },
      { id: 'grok-3-latest', name: 'Grok 3 Latest' },
    ],
  },
  {
    id: 'opencode',
    name: 'OpenCode',
    models: [
      { id: 'claude-sonnet-4-5-20250929', name: 'Claude Sonnet 4.5' },
      { id: 'gpt-5-codex', name: 'GPT-5 Codex' },
      { id: 'gemini-2.5-pro', name: 'Gemini 2.5 Pro' },
      { id: 'grok-4-latest', name: 'Grok 4 Latest' },
    ],
  },
]

interface AgentModelSelectorProps {
  agentId: string | null
  model: string | null
  onAgentChange: (agentId: string | null) => void
  onModelChange: (model: string | null) => void
}

export function AgentModelSelector({
  agentId,
  model,
  onAgentChange,
  onModelChange,
}: AgentModelSelectorProps) {
  const selectedAgent = AGENTS.find((a) => a.id === agentId)
  const availableModels = selectedAgent?.models || []

  const handleAgentChange = (value: string) => {
    const newAgentId = value === 'none' ? null : value
    onAgentChange(newAgentId)

    // Reset model if not available for new agent
    if (newAgentId) {
      const newAgent = AGENTS.find((a) => a.id === newAgentId)
      const modelAvailable = newAgent?.models.some((m) => m.id === model)
      if (!modelAvailable) {
        onModelChange(newAgent?.models[0]?.id || null)
      }
    } else {
      onModelChange(null)
    }
  }

  const handleModelChange = (value: string) => {
    onModelChange(value === 'none' ? null : value)
  }

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <Label>AI Agent (Optional)</Label>
        <Select value={agentId || 'none'} onValueChange={handleAgentChange}>
          <SelectTrigger>
            <SelectValue placeholder="Select an agent..." />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="none">No agent</SelectItem>
            {AGENTS.map((agent) => (
              <SelectItem key={agent.id} value={agent.id}>
                {agent.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <p className="text-sm text-muted-foreground">
          Choose an AI agent to assist with code execution
        </p>
      </div>

      {agentId && (
        <div className="space-y-2">
          <Label>Model</Label>
          <Select value={model || 'none'} onValueChange={handleModelChange}>
            <SelectTrigger>
              <SelectValue placeholder="Select a model..." />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="none">Default model</SelectItem>
              {availableModels.map((m) => (
                <SelectItem key={m.id} value={m.id}>
                  {m.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <p className="text-sm text-muted-foreground">
            Select a specific model for this agent
          </p>
        </div>
      )}
    </div>
  )
}

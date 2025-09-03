import { useLocation, Link } from 'react-router-dom'
import {
  Breadcrumb,
  BreadcrumbList,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbSeparator,
  BreadcrumbPage,
} from '@/components/ui/breadcrumb'

const routeLabels: Record<string, string> = {
  '/': 'Usage',
  '/projects': 'Projects',
  '/ai-chat': 'AI Chat',
  '/mcp-servers': 'MCP Servers',
  '/monitoring': 'Monitoring',
  '/settings': 'Settings'
}

export function Breadcrumbs() {
  const location = useLocation()
  const currentLabel = routeLabels[location.pathname] || 'Dashboard'

  return (
    <Breadcrumb>
      <BreadcrumbList>
        <BreadcrumbItem className="hidden md:block">
          <BreadcrumbLink asChild>
            <Link to="/">Orkee Dashboard</Link>
          </BreadcrumbLink>
        </BreadcrumbItem>
        <BreadcrumbSeparator className="hidden md:block" />
        <BreadcrumbItem>
          <BreadcrumbPage>{currentLabel}</BreadcrumbPage>
        </BreadcrumbItem>
      </BreadcrumbList>
    </Breadcrumb>
  )
}
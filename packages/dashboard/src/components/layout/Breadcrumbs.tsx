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
  '/projects': 'Projects',
  '/settings': 'Settings'
}

export function Breadcrumbs() {
  const location = useLocation()
  
  // Check if we're on a project detail page
  const isProjectDetail = location.pathname.startsWith('/projects/') && location.pathname !== '/projects'
  
  // Handle dynamic routes like /projects/[id]
  let currentLabel = routeLabels[location.pathname] || 'Dashboard'
  if (location.pathname.startsWith('/projects/')) {
    currentLabel = 'Projects'
  }

  return (
    <Breadcrumb>
      <BreadcrumbList>
        <BreadcrumbItem className="hidden md:block">
          <BreadcrumbLink asChild>
            <Link to="/">Orkee</Link>
          </BreadcrumbLink>
        </BreadcrumbItem>
        <BreadcrumbSeparator className="hidden md:block" />
        <BreadcrumbItem>
          {isProjectDetail ? (
            <BreadcrumbLink asChild>
              <Link to="/projects">Projects</Link>
            </BreadcrumbLink>
          ) : (
            <BreadcrumbPage>{currentLabel}</BreadcrumbPage>
          )}
        </BreadcrumbItem>
      </BreadcrumbList>
    </Breadcrumb>
  )
}
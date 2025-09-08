import { Link, useLocation } from 'react-router-dom'
import { 
  FolderOpen,
  Settings,
  Menu,
  X
} from 'lucide-react'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { useState } from 'react'

interface SidebarItem {
  title: string
  href: string
  icon: React.ComponentType<React.SVGProps<SVGSVGElement>>
}

const sidebarItems: SidebarItem[] = [
  {
    title: "Projects",
    href: "/projects",
    icon: FolderOpen
  },
  {
    title: "Settings",
    href: "/settings",
    icon: Settings
  }
]

interface SidebarProps {
  className?: string
}

export function Sidebar({ className }: SidebarProps) {
  const location = useLocation()
  const [isCollapsed, setIsCollapsed] = useState(false)

  return (
    <div className={cn(
      "relative flex flex-col border-r bg-background",
      isCollapsed ? "w-16" : "w-64",
      className
    )}>
      {/* Header */}
      <div className="flex h-14 items-center justify-between px-4 border-b">
        <h1 className={cn(
          "font-semibold text-lg",
          isCollapsed && "hidden"
        )}>
          Orkee
        </h1>
        <Button
          variant="ghost"
          size="icon"
          onClick={() => setIsCollapsed(!isCollapsed)}
          className="h-8 w-8"
        >
          {isCollapsed ? <Menu className="h-4 w-4" /> : <X className="h-4 w-4" />}
        </Button>
      </div>

      {/* Navigation */}
      <nav className="flex-1 space-y-1 p-4">
        {sidebarItems.map((item) => {
          const Icon = item.icon
          const isActive = location.pathname === item.href || 
            (item.href !== "/" && location.pathname.startsWith(item.href))
          
          return (
            <Link
              key={item.href}
              to={item.href}
              className={cn(
                "flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
                "hover:bg-accent hover:text-accent-foreground",
                isActive 
                  ? "bg-accent text-accent-foreground" 
                  : "text-muted-foreground"
              )}
            >
              <Icon className="h-4 w-4" />
              {!isCollapsed && (
                <span>{item.title}</span>
              )}
            </Link>
          )
        })}
      </nav>
    </div>
  )
}
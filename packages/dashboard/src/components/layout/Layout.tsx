import { ReactNode, useState, useEffect } from 'react'
import { AppSidebar } from "@/components/app-sidebar"
import { Breadcrumbs } from './Breadcrumbs'
import { CloudAuthButtonHeader } from '@/components/cloud/CloudAuthButton'
import { ThemeSwitcher } from '@/components/ThemeSwitcher'
import { fetchConfig } from '@/services/config'
import { Separator } from "@/components/ui/separator"
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar"

interface LayoutProps {
  children: ReactNode
}

export function Layout({ children }: LayoutProps) {
  const [isCloudEnabled, setIsCloudEnabled] = useState(false)

  useEffect(() => {
    fetchConfig().then(config => {
      setIsCloudEnabled(config.cloud_enabled)
    })
  }, [])

  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <header className="flex h-16 shrink-0 items-center gap-2 transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-12">
          <div className="flex items-center gap-2 px-4">
            <SidebarTrigger className="-ml-1" />
            <Separator
              orientation="vertical"
              className="mr-2 data-[orientation=vertical]:h-4"
            />
            <Breadcrumbs />
          </div>
          <div className="ml-auto flex items-center gap-2 px-4">
            <ThemeSwitcher />
            {isCloudEnabled && <CloudAuthButtonHeader />}
          </div>
        </header>
        <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
          {children}
        </div>
      </SidebarInset>
    </SidebarProvider>
  )
}
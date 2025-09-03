"use client"

import * as React from "react"
import {
  TrendingUp,
  FolderOpen,
  MessageSquare,
  Server,
  Monitor,
  Settings,
  Bot,
  Activity,
  Database,
} from "lucide-react"

import { NavMain } from "@/components/nav-main"
import { NavProjects } from "@/components/nav-projects"
import { NavUser } from "@/components/nav-user"
import { TeamSwitcher } from "@/components/team-switcher"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from "@/components/ui/sidebar"

// Orkee navigation data
const data = {
  user: {
    name: "Admin",
    email: "admin@orkee.ai",
    avatar: "/avatars/admin.jpg",
  },
  teams: [
    {
      name: "Orkee",
      logo: Bot,
      plan: "Enterprise",
    },
  ],
  navMain: [
    {
      title: "Usage",
      url: "/",
      icon: TrendingUp,
      isActive: true,
    },
    {
      title: "Projects", 
      url: "/projects",
      icon: FolderOpen,
    },
    {
      title: "AI Chat",
      url: "/ai-chat", 
      icon: MessageSquare,
    },
    {
      title: "MCP Servers",
      url: "/mcp-servers",
      icon: Server,
    },
    {
      title: "Monitoring",
      url: "/monitoring",
      icon: Monitor,
    },
    {
      title: "Settings",
      url: "/settings",
      icon: Settings,
    },
  ],
  projects: [
    {
      name: "Customer Support Bot",
      url: "#",
      icon: MessageSquare,
    },
    {
      name: "Data Processing Pipeline", 
      url: "#",
      icon: Database,
    },
    {
      name: "Analytics Dashboard",
      url: "#", 
      icon: Activity,
    },
  ],
}

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader>
        <TeamSwitcher teams={data.teams} />
      </SidebarHeader>
      <SidebarContent>
        <NavMain items={data.navMain} />
        <NavProjects projects={data.projects} />
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={data.user} />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}
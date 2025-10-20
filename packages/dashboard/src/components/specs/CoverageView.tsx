// ABOUTME: Coverage view showing spec-to-task coverage metrics and orphan detection
// ABOUTME: Integrates SyncDashboard for comprehensive coverage analysis
import { SyncDashboard } from '@/components/SyncDashboard';

interface CoverageViewProps {
  projectId: string;
}

export function CoverageView({ projectId }: CoverageViewProps) {
  return <SyncDashboard projectId={projectId} />;
}

import { useSidebar } from '@/hooks/useSidebar';
import { ConnectionStatus } from './ConnectionStatus';

export function SidebarConnectionStatus() {
  const { state } = useSidebar();
  
  return (
    <div className="p-2">
      <ConnectionStatus showText={state !== "collapsed"} />
    </div>
  );
}
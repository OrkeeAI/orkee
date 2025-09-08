import React, { useMemo } from 'react';
import { CommitInfo } from '../../services/git';
import {
  ContributionGraph,
  ContributionGraphCalendar,
  ContributionGraphBlock,
  ContributionGraphFooter,
  ContributionGraphTotalCount,
  ContributionGraphLegend,
  type Activity,
} from '@/components/ui/kibo-ui/contribution-graph';
import { cn } from '@/lib/utils';

interface GitActivityGraphProps {
  commits: CommitInfo[];
  className?: string;
}

export const GitActivityGraph: React.FC<GitActivityGraphProps> = ({
  commits,
  className = '',
}) => {
  // Transform commit data into activity graph format
  const activityData = useMemo((): Activity[] => {
    // Calculate date range (last 12 months)
    const endDate = new Date();
    const startDate = new Date();
    startDate.setFullYear(endDate.getFullYear() - 1);
    startDate.setDate(startDate.getDate() + 1); // Start from next day to make it exactly 12 months

    // Create a map of all dates in the range
    const dateMap = new Map<string, { commits: CommitInfo[]; count: number }>();
    const currentDate = new Date(startDate);
    
    while (currentDate <= endDate) {
      const dateStr = currentDate.toISOString().split('T')[0];
      dateMap.set(dateStr, { commits: [], count: 0 });
      currentDate.setDate(currentDate.getDate() + 1);
    }

    // Group commits by date
    commits.forEach((commit) => {
      const commitDate = new Date(commit.timestamp * 1000);
      const dateKey = commitDate.toISOString().split('T')[0];
      
      if (dateMap.has(dateKey)) {
        const dayData = dateMap.get(dateKey)!;
        dayData.commits.push(commit);
        dayData.count++;
      }
    });

    // Convert to Activity format with levels
    const activities: Activity[] = [];
    
    dateMap.forEach((dayData, date) => {
      const count = dayData.count;
      
      // Determine intensity level based on commit count (0-4 scale)
      let level = 0;
      if (count >= 8) level = 4;
      else if (count >= 5) level = 3;
      else if (count >= 3) level = 2;
      else if (count >= 1) level = 1;

      activities.push({
        date,
        count,
        level,
      });
    });

    return activities.sort((a, b) => a.date.localeCompare(b.date));
  }, [commits]);

  if (commits.length === 0) {
    return (
      <div className="flex items-center justify-center h-32 text-muted-foreground">
        <div className="text-center">
          <p className="text-sm">No commit activity found</p>
          <p className="text-xs text-muted-foreground">
            Make some commits to see your activity graph
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className={`${className} w-full`}>
      <ContributionGraph
        data={activityData}
        maxLevel={4}
        blockSize={12}
        blockRadius={2}
        blockMargin={3}
        fontSize={12}
        labels={{
          months: ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"],
          legend: {
            less: "Less",
            more: "More",
          },
        }}
        className="w-full"
      >
        <ContributionGraphCalendar>
          {({ activity, dayIndex, weekIndex }) => (
            <ContributionGraphBlock
              activity={activity}
              dayIndex={dayIndex}
              weekIndex={weekIndex}
              className={cn(
                'data-[level="0"]:fill-[#ebedf0] dark:data-[level="0"]:fill-[#161b22]',
                'data-[level="1"]:fill-[#9be9a8] dark:data-[level="1"]:fill-[#0e4429]',
                'data-[level="2"]:fill-[#40c463] dark:data-[level="2"]:fill-[#006d32]',
                'data-[level="3"]:fill-[#30a14e] dark:data-[level="3"]:fill-[#26a641]',
                'data-[level="4"]:fill-[#216e39] dark:data-[level="4"]:fill-[#39d353]',
                "transition-all duration-200 hover:stroke-border hover:stroke-2"
              )}
              style={{
                cursor: 'default',
              }}
            />
          )}
        </ContributionGraphCalendar>
        
        <ContributionGraphFooter className="justify-between w-full">
          <ContributionGraphTotalCount>
            {({ totalCount }) => (
              <div className="text-muted-foreground text-sm">
                {totalCount} commits in the last year
              </div>
            )}
          </ContributionGraphTotalCount>
          <ContributionGraphLegend>
            {({ level }) => (
              <svg height={12} width={12}>
                <title>{`${level} contributions`}</title>
                <rect
                  className={cn(
                    "stroke-[1px] stroke-border",
                    'data-[level="0"]:fill-[#ebedf0] dark:data-[level="0"]:fill-[#161b22]',
                    'data-[level="1"]:fill-[#9be9a8] dark:data-[level="1"]:fill-[#0e4429]',
                    'data-[level="2"]:fill-[#40c463] dark:data-[level="2"]:fill-[#006d32]',
                    'data-[level="3"]:fill-[#30a14e] dark:data-[level="3"]:fill-[#26a641]',
                    'data-[level="4"]:fill-[#216e39] dark:data-[level="4"]:fill-[#39d353]'
                  )}
                  data-level={level}
                  height={12}
                  rx={2}
                  ry={2}
                  width={12}
                />
              </svg>
            )}
          </ContributionGraphLegend>
        </ContributionGraphFooter>
      </ContributionGraph>
    </div>
  );
};
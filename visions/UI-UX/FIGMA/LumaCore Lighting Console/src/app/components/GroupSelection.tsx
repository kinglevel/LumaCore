import React from "react";
import { Users, Star } from "lucide-react";

interface Group {
  id: string;
  number: number;
  name: string;
  fixtureCount: number;
  color?: string;
}

interface GroupSelectionProps {
  onGroupSelect?: (groupId: string) => void;
  selectedGroup?: string;
}

export function GroupSelection({ onGroupSelect, selectedGroup }: GroupSelectionProps) {
  const groups: Group[] = [
    { id: 'group-1', number: 1, name: 'Moving Heads', fixtureCount: 2, color: '#d4a944' },
    { id: 'group-2', number: 2, name: 'Stage Wash', fixtureCount: 2, color: '#4a9e5c' },
    { id: 'group-3', number: 3, name: 'Front PARs', fixtureCount: 2, color: '#6e8eb8' },
    { id: 'group-4', number: 4, name: 'LED Bars', fixtureCount: 2, color: '#9a6ec4' },
    { id: 'group-5', number: 5, name: 'All Fixtures', fixtureCount: 10, color: '#c4c4c8' },
  ];

  return (
    <div className="border border-border bg-card">
      <div className="border-b border-border bg-secondary/30 px-4 py-2">
        <div className="flex items-center gap-2">
          <Users className="w-3.5 h-3.5 text-primary" />
          <span className="text-xs uppercase tracking-wider text-foreground">Groups</span>
        </div>
      </div>

      <div className="p-3 bg-secondary/5">
        <div className="grid grid-cols-1 gap-2">
          {groups.map((group) => {
            const isSelected = selectedGroup === group.id;
            
            return (
              <button
                key={group.id}
                onClick={() => onGroupSelect?.(group.id)}
                className={`
                  flex items-center justify-between px-3 py-2 border transition-all text-left
                  ${isSelected
                    ? 'border-primary bg-primary/20 text-primary'
                    : 'border-border bg-card text-foreground hover:bg-secondary'
                  }
                `}
              >
                <div className="flex items-center gap-2">
                  <div 
                    className="w-2 h-2 rounded-full"
                    style={{ backgroundColor: group.color }}
                  />
                  <div>
                    <div className="text-xs font-mono">{group.number}</div>
                  </div>
                  <div className="text-xs">{group.name}</div>
                </div>
                <div className="text-[10px] text-muted-foreground">
                  {group.fixtureCount} fix
                </div>
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}

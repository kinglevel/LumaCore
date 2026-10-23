import React, { useState } from "react";
import { Users, Plus, Edit2, Trash2 } from "lucide-react";

interface Group {
  id: string;
  number: number;
  name: string;
  fixtureCount: number;
  color?: string;
}

export default function Groups() {
  const [selectedGroup, setSelectedGroup] = useState<string | null>(null);
  
  const groups: Group[] = [
    { id: 'group-1', number: 1, name: 'Moving Heads', fixtureCount: 2, color: '#d4a944' },
    { id: 'group-2', number: 2, name: 'Stage Wash', fixtureCount: 2, color: '#4a9e5c' },
    { id: 'group-3', number: 3, name: 'Front PARs', fixtureCount: 2, color: '#6e8eb8' },
    { id: 'group-4', number: 4, name: 'LED Bars', fixtureCount: 2, color: '#9a6ec4' },
    { id: 'group-5', number: 5, name: 'All Fixtures', fixtureCount: 10, color: '#c4c4c8' },
    { id: 'group-6', number: 6, name: 'Downstage', fixtureCount: 4, color: '#d47444' },
    { id: 'group-7', number: 7, name: 'Upstage', fixtureCount: 3, color: '#7444d4' },
    { id: 'group-8', number: 8, name: 'Side Light', fixtureCount: 4, color: '#44d4a9' },
  ];

  return (
    <div className="flex flex-col h-full p-4">
      {/* Header with Stats */}
      <div className="border border-border bg-card mb-4 flex-shrink-0">
        <div className="border-b border-border bg-secondary/30 px-4 py-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-1.5 h-1.5 rounded-full bg-primary" />
              <span className="text-xs uppercase tracking-wider text-foreground">Group Management</span>
            </div>
            <button className="flex items-center gap-2 px-3 py-1 border border-border bg-card hover:bg-secondary transition-colors text-xs">
              <Plus className="w-3.5 h-3.5" />
              <span className="uppercase tracking-wide">New Group</span>
            </button>
          </div>
        </div>
        <div className="grid grid-cols-3 divide-x divide-border">
          <div className="p-4 bg-secondary/10">
            <div className="text-2xl font-mono text-primary mb-1">{groups.length}</div>
            <div className="text-xs text-muted-foreground uppercase tracking-wide">Total Groups</div>
          </div>
          <div className="p-4 bg-secondary/10">
            <div className="text-2xl font-mono text-primary mb-1">
              {selectedGroup ? '1' : '0'}
            </div>
            <div className="text-xs text-muted-foreground uppercase tracking-wide">Selected</div>
          </div>
          <div className="p-4 bg-secondary/10">
            <div className="text-2xl font-mono text-primary mb-1">
              {groups.reduce((sum, g) => sum + g.fixtureCount, 0)}
            </div>
            <div className="text-xs text-muted-foreground uppercase tracking-wide">Total Fixtures</div>
          </div>
        </div>
      </div>

      {/* Groups Grid */}
      <div className="border border-border bg-card flex-1 flex flex-col min-h-0">
        <div className="border-b border-border bg-secondary/30 px-4 py-2">
          <div className="flex items-center gap-2">
            <Users className="w-3.5 h-3.5 text-primary" />
            <span className="text-xs uppercase tracking-wider text-foreground">Group Library</span>
          </div>
        </div>

        <div className="p-4 bg-secondary/5 flex-1 overflow-y-auto">
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 gap-3">
            {groups.map((group) => {
              const isSelected = selectedGroup === group.id;
              
              return (
                <button
                  key={group.id}
                  onClick={() => setSelectedGroup(isSelected ? null : group.id)}
                  className={`
                    relative flex flex-col items-center justify-center p-4 border transition-all
                    aspect-square
                    ${isSelected
                      ? 'border-primary bg-primary/20'
                      : 'border-border bg-card hover:bg-secondary hover:border-primary/50'
                    }
                  `}
                >
                  {/* Group Indicator */}
                  <div 
                    className={`w-12 h-12 rounded-full mb-3 flex items-center justify-center transition-all ${
                      isSelected ? 'scale-110' : ''
                    }`}
                    style={{ backgroundColor: group.color }}
                  >
                    <span className="text-xl font-mono font-bold text-background/90">
                      {group.number}
                    </span>
                  </div>
                  
                  {/* Group Name */}
                  <div className="text-sm font-medium text-foreground text-center mb-1 truncate w-full px-2">
                    {group.name}
                  </div>
                  
                  {/* Fixture Count */}
                  <div className="text-xs text-muted-foreground">
                    {group.fixtureCount} fixtures
                  </div>

                  {/* Selection Indicator */}
                  {isSelected && (
                    <div className="absolute top-2 right-2">
                      <div className="w-2 h-2 rounded-full bg-primary animate-pulse" />
                    </div>
                  )}

                  {/* Hover Actions */}
                  <div className="absolute top-2 left-2 opacity-0 hover:opacity-100 transition-opacity flex gap-1">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        // Edit action
                      }}
                      className="p-1 bg-secondary/90 hover:bg-primary/20 border border-border rounded"
                    >
                      <Edit2 className="w-3 h-3 text-foreground" />
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        // Delete action
                      }}
                      className="p-1 bg-secondary/90 hover:bg-[#c93838]/20 border border-border rounded"
                    >
                      <Trash2 className="w-3 h-3 text-[#c93838]" />
                    </button>
                  </div>
                </button>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}

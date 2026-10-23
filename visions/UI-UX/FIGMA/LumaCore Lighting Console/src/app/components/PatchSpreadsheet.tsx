import React from "react";
import { useState } from "react";
import { Fixture } from "../types/lighting";
import { Check, X, Edit2 } from "lucide-react";

interface PatchSpreadsheetProps {
  fixtures: Fixture[];
  onFixtureUpdate?: (id: string, updates: Partial<Fixture>) => void;
  selectedFixtures?: string[];
  onSelectionChange?: (ids: string[]) => void;
}

export function PatchSpreadsheet({ 
  fixtures, 
  onFixtureUpdate,
  selectedFixtures = [],
  onSelectionChange 
}: PatchSpreadsheetProps) {
  const [editingCell, setEditingCell] = useState<{ id: string; field: string } | null>(null);

  const handleRowClick = (id: string, e: React.MouseEvent) => {
    if (e.ctrlKey || e.metaKey) {
      // Multi-select
      if (selectedFixtures.includes(id)) {
        onSelectionChange?.(selectedFixtures.filter(fId => fId !== id));
      } else {
        onSelectionChange?.([...selectedFixtures, id]);
      }
    } else {
      // Single select
      onSelectionChange?.([id]);
    }
  };

  return (
    <div className="border border-border bg-card overflow-hidden">
      <div className="border-b border-border bg-secondary/30 px-4 py-2">
        <div className="flex items-center gap-2">
          <div className="w-1.5 h-1.5 rounded-full bg-primary" />
          <span className="text-xs uppercase tracking-wider text-foreground">Fixture Patch Sheet</span>
        </div>
      </div>

      {/* Spreadsheet */}
      <div className="overflow-x-auto">
        <table className="w-full text-xs">
          {/* Header */}
          <thead className="sticky top-0 bg-secondary/40 border-b-2 border-border">
            <tr>
              <th className="px-3 py-2 text-left font-medium text-muted-foreground uppercase tracking-wide border-r border-border">
                #
              </th>
              <th className="px-3 py-2 text-left font-medium text-muted-foreground uppercase tracking-wide border-r border-border">
                Chan
              </th>
              <th className="px-3 py-2 text-left font-medium text-muted-foreground uppercase tracking-wide border-r border-border min-w-[150px]">
                Fixture Name
              </th>
              <th className="px-3 py-2 text-left font-medium text-muted-foreground uppercase tracking-wide border-r border-border min-w-[140px]">
                Type
              </th>
              <th className="px-3 py-2 text-left font-medium text-muted-foreground uppercase tracking-wide border-r border-border min-w-[140px]">
                Manufacturer
              </th>
              <th className="px-3 py-2 text-left font-medium text-muted-foreground uppercase tracking-wide border-r border-border">
                Uni
              </th>
              <th className="px-3 py-2 text-left font-medium text-muted-foreground uppercase tracking-wide border-r border-border">
                DMX Addr
              </th>
              <th className="px-3 py-2 text-left font-medium text-muted-foreground uppercase tracking-wide border-r border-border">
                Ch Count
              </th>
              <th className="px-3 py-2 text-left font-medium text-muted-foreground uppercase tracking-wide border-r border-border">
                Int
              </th>
              <th className="px-3 py-2 text-left font-medium text-muted-foreground uppercase tracking-wide">
                Status
              </th>
            </tr>
          </thead>

          {/* Body */}
          <tbody className="divide-y divide-border">
            {fixtures.map((fixture, index) => {
              const isSelected = selectedFixtures.includes(fixture.id);
              const isActive = fixture.intensity > 0;
              
              return (
                <tr
                  key={fixture.id}
                  onClick={(e) => handleRowClick(fixture.id, e)}
                  className={`
                    transition-colors cursor-pointer
                    ${isSelected 
                      ? 'bg-primary/15 hover:bg-primary/20' 
                      : 'bg-card hover:bg-secondary/30'
                    }
                  `}
                >
                  <td className="px-3 py-2 border-r border-border font-mono text-muted-foreground">
                    {(index + 1).toString().padStart(3, '0')}
                  </td>
                  <td className="px-3 py-2 border-r border-border font-mono text-primary">
                    {(index + 1).toString().padStart(3, '0')}
                  </td>
                  <td className="px-3 py-2 border-r border-border font-mono text-foreground">
                    {fixture.name}
                  </td>
                  <td className="px-3 py-2 border-r border-border text-foreground">
                    {fixture.type}
                  </td>
                  <td className="px-3 py-2 border-r border-border text-muted-foreground">
                    {fixture.manufacturer}
                  </td>
                  <td className="px-3 py-2 border-r border-border font-mono text-center text-foreground">
                    {fixture.universe}
                  </td>
                  <td className="px-3 py-2 border-r border-border font-mono text-primary">
                    {fixture.dmxAddress}
                  </td>
                  <td className="px-3 py-2 border-r border-border font-mono text-center text-foreground">
                    {fixture.channels}
                  </td>
                  <td className="px-3 py-2 border-r border-border font-mono text-primary text-center">
                    {Math.round((fixture.intensity / 255) * 100)}%
                  </td>
                  <td className="px-3 py-2">
                    <div className="flex items-center justify-center">
                      {isActive ? (
                        <Check className="w-3.5 h-3.5 text-[#4a9e5c]" />
                      ) : (
                        <X className="w-3.5 h-3.5 text-muted-foreground" />
                      )}
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      {/* Footer stats */}
      <div className="border-t border-border bg-secondary/20 px-4 py-2">
        <div className="flex items-center justify-between text-xs">
          <div className="flex items-center gap-4">
            <span className="text-muted-foreground">
              Total: <span className="font-mono text-foreground">{fixtures.length}</span> fixtures
            </span>
            <span className="text-muted-foreground">
              Selected: <span className="font-mono text-primary">{selectedFixtures.length}</span>
            </span>
          </div>
          <div className="font-mono text-muted-foreground">
            DMX Footprint: {fixtures.reduce((sum, f) => sum + f.channels, 0)}/512
          </div>
        </div>
      </div>
    </div>
  );
}

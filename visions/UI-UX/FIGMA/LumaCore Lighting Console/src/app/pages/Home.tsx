import React from "react";
import { useState } from "react";
import { Circle } from "lucide-react";

export default function Home() {
  const [selectedTab, setSelectedTab] = useState<string>('fixturetype');
  const [selectedCells, setSelectedCells] = useState<string[]>([]);

  // Fixture data matching the image
  const fixtures = [
    { name: 'Alpha 1500 Profile', lx1: '6', lx2: '6', lx3: '', lx4: '24', lx5: '', lx6: '', lx7: '', lx8: '', lx9: '', lx10: '', lx11: '', lx12: '24', lx13: '' },
    { name: 'Alpha 700 Hipe', lx1: '6', lx2: '', lx3: '', lx4: '', lx5: '8', lx6: '24', lx7: '', lx8: '', lx9: '', lx10: '', lx11: '', lx12: '', lx13: '' },
    { name: 'Mac Aura', lx1: '', lx2: '', lx3: '', lx4: '', lx5: '8', lx6: '', lx7: '10', lx8: '', lx9: '', lx10: '', lx11: '24', lx12: '', lx13: '6' },
    { name: 'VL 3500 W AirFX', lx1: '', lx2: '', lx3: '', lx4: '', lx5: '12', lx6: '', lx7: '', lx8: '', lx9: '', lx10: '', lx11: '', lx12: '', lx13: '' },
    { name: 'Atomic 3000', lx1: '6', lx2: '', lx3: '6', lx4: '', lx5: '', lx6: '6', lx7: '', lx8: '6', lx9: '', lx10: '6', lx11: '', lx12: '6', lx13: '6' },
    { name: 'Source 4', lx1: '', lx2: '10', lx3: '', lx4: '', lx5: '', lx6: '', lx7: '10', lx8: '', lx9: '', lx10: '', lx11: '', lx12: '', lx13: '' },
    { name: 'EOS 100', lx1: '', lx2: '', lx3: '', lx4: '', lx5: '8', lx6: '24', lx7: '', lx8: '', lx9: '', lx10: '', lx11: '', lx12: '', lx13: '' },
    { name: 'K20 B-Eye', lx1: '', lx2: '', lx3: '', lx4: '', lx5: '6', lx6: '', lx7: '', lx8: '', lx9: '', lx10: '', lx11: '8', lx12: '', lx13: '' },
    { name: 'Mythos', lx1: '', lx2: '', lx3: '', lx4: '', lx5: '', lx6: '', lx7: '', lx8: '6', lx9: '', lx10: '', lx11: '', lx12: '', lx13: '' },
    { name: 'Sharpy', lx1: '', lx2: '10', lx3: '', lx4: '24', lx5: '', lx6: '', lx7: '', lx8: '', lx9: '', lx10: '', lx11: '', lx12: '', lx13: '' },
  ];

  const columns = ['Lx1', 'Lx2', 'Lx3', 'Lx4', 'Lx5', 'Lx6', 'Lx7', 'Lx8', 'Lx9', 'Lx10', 'Lx11', 'Lx12', 'Lx13', '...'];

  return (
    <div className="flex flex-col h-full bg-background">
      {/* Top Status Bar */}
      <div className="flex-shrink-0 bg-[#1a1a1a] border-b border-[#3a3a3a] px-4 py-1.5 flex items-center justify-between">
        <div className="flex items-center gap-6 text-xs">
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 rounded-full bg-primary" />
            <span className="text-foreground font-medium">ShowInfo</span>
          </div>
          <span className="text-muted-foreground">Firmware: X.X.X.X</span>
          <span className="text-muted-foreground">ShowFile: BigAssShow</span>
          <span className="text-muted-foreground">Rev: 1.1</span>
          <span className="text-muted-foreground">User: Administrator</span>
          <span className="text-muted-foreground">World: Full</span>
          <span className="text-muted-foreground">Zone: Full</span>
          <span className="text-muted-foreground">SoundOut: Headphones</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-xs text-muted-foreground font-mono">2020-01-01 00:00</span>
          <div className="w-2 h-2 rounded-full bg-[#4a9e5c]" />
        </div>
      </div>

      {/* Main Content Area */}
      <div className="flex-1 flex min-h-0">
        {/* Left Sidebar */}
        <div className="w-[380px] flex-shrink-0 border-r border-border flex flex-col gap-4 p-4 bg-background">
          {/* Positions/Shapers Panel */}
          <div className="border border-border bg-card">
            <div className="border-b border-border bg-secondary/30 px-3 py-1.5 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="flex items-center gap-1.5">
                  <div className="w-1.5 h-1.5 rounded-full bg-primary" />
                  <span className="text-[10px] uppercase tracking-wider text-foreground font-medium">Positions</span>
                </div>
                <span className="text-[10px] uppercase tracking-wider text-muted-foreground">Shapers</span>
              </div>
            </div>
            <div className="p-2 bg-secondary/5">
              <div className="mb-2">
                <div className="flex items-center gap-1.5 mb-1">
                  <Circle className="w-2.5 h-2.5 text-muted-foreground" />
                  <span className="text-[10px] text-muted-foreground uppercase">Preset Sort</span>
                </div>
              </div>
              <div className="grid grid-cols-6 gap-1">
                {Array.from({ length: 18 }).map((_, i) => (
                  <div key={i} className="aspect-square bg-[#2a2a2a] border border-[#3a3a3a] rounded-sm" />
                ))}
              </div>
            </div>
          </div>

          {/* Groups Panel */}
          <div className="border border-border bg-card flex-1 flex flex-col min-h-0">
            <div className="border-b border-border bg-secondary/30 px-3 py-1.5 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="flex items-center gap-1.5">
                  <div className="w-1.5 h-1.5 rounded-full bg-primary" />
                  <span className="text-[10px] uppercase tracking-wider text-foreground font-medium">Groups</span>
                </div>
                <span className="text-[10px] uppercase tracking-wider text-muted-foreground">Groups</span>
                <span className="text-[10px] uppercase tracking-wider text-muted-foreground">Groups</span>
                <span className="text-[10px] uppercase tracking-wider text-primary">Layout</span>
              </div>
            </div>
            <div className="p-2 bg-secondary/5 flex-1 overflow-y-auto">
              <div className="mb-2">
                <div className="flex items-center gap-1.5 mb-1">
                  <Circle className="w-2.5 h-2.5 text-muted-foreground" />
                  <span className="text-[10px] text-muted-foreground uppercase">Icons</span>
                </div>
              </div>
              <div className="grid grid-cols-6 gap-1">
                {Array.from({ length: 60 }).map((_, i) => (
                  <div key={i} className="aspect-square bg-[#2a2a2a] border border-[#3a3a3a] rounded-sm" />
                ))}
              </div>
            </div>
          </div>

          {/* Executors Panel */}
          <div className="border border-border bg-card">
            <div className="border-b border-border bg-secondary/30 px-3 py-1.5 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="flex items-center gap-1.5">
                  <div className="w-1.5 h-1.5 rounded-full bg-primary" />
                  <span className="text-[10px] uppercase tracking-wider text-foreground font-medium">Executors</span>
                </div>
                <span className="text-[10px] uppercase tracking-wider text-muted-foreground">Attributes</span>
              </div>
            </div>
            <div className="p-2 bg-secondary/5">
              {/* Executor rows */}
              <div className="space-y-2">
                {['Ratemaster', 'RateMstr+'].map((label, rowIdx) => (
                  <div key={rowIdx}>
                    <div className="text-[9px] text-muted-foreground mb-1 uppercase">{label}</div>
                    <div className="grid grid-cols-6 gap-1">
                      {Array.from({ length: 6 }).map((_, i) => (
                        <div key={i} className="border border-[#3a3a3a] bg-[#1a1a1a] rounded-sm p-1.5">
                          <div className="text-[8px] text-muted-foreground mb-0.5">
                            {rowIdx === 0 ? 'Exec/Mstr' : 'Fader/Mstr'}
                          </div>
                          <div className="h-8 bg-[#2a2a2a] rounded-sm" />
                        </div>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>

        {/* Right Main Panel - Spreadsheet */}
        <div className="flex-1 flex flex-col min-h-0 bg-background">
          {/* Tab Bar */}
          <div className="flex-shrink-0 border-b border-border bg-[#1a1a1a] px-3 py-1 flex items-center gap-1">
            {[
              { id: 'fixturetype', label: 'FixtureType Sheet', active: true },
              { id: 'channel', label: 'Channel' },
              { id: 'fixture', label: 'Fixture' },
              { id: 'dmx', label: 'DMX' },
              { id: 'sequence', label: 'Sequence Exec' },
            ].map((tab) => (
              <button
                key={tab.id}
                onClick={() => setSelectedTab(tab.id)}
                className={`px-3 py-1 text-[10px] uppercase tracking-wide transition-colors ${
                  selectedTab === tab.id || tab.active
                    ? 'bg-primary text-primary-foreground'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
              >
                <div className="flex items-center gap-1.5">
                  {(selectedTab === tab.id || tab.active) && (
                    <div className="w-1.5 h-1.5 rounded-full bg-primary-foreground" />
                  )}
                  {tab.label}
                </div>
              </button>
            ))}
          </div>

          {/* Toolbar */}
          <div className="flex-shrink-0 border-b border-border bg-[#1a1a1a] px-3 py-1.5 flex items-center gap-4">
            <div className="flex items-center gap-2">
              <Circle className="w-2.5 h-2.5 text-primary" />
              <span className="text-[10px] uppercase tracking-wide text-foreground">Direct Select</span>
            </div>
            <div className="flex items-center gap-2">
              <Circle className="w-2.5 h-2.5 text-muted-foreground" />
              <span className="text-[10px] uppercase tracking-wide text-muted-foreground">Fixture Sort</span>
            </div>
            <div className="flex items-center gap-2">
              <Circle className="w-2.5 h-2.5 text-muted-foreground" />
              <span className="text-[10px] uppercase tracking-wide text-muted-foreground">Mask</span>
            </div>
            <div className="flex items-center gap-2">
              <Circle className="w-2.5 h-2.5 text-muted-foreground" />
              <span className="text-[10px] uppercase tracking-wide text-muted-foreground">Icons</span>
            </div>
            <div className="flex items-center gap-2">
              <Circle className="w-2.5 h-2.5 text-muted-foreground" />
              <span className="text-[10px] uppercase tracking-wide text-muted-foreground">Sort by: Trust</span>
            </div>
          </div>

          {/* Attribute Buttons */}
          <div className="flex-shrink-0 border-b border-border bg-[#1a1a1a] px-3 py-1.5 flex items-center gap-2">
            {['Dimmer', 'Position', 'Gobo', 'Beam', 'Focus', 'Control', 'Shapers'].map((attr, idx) => (
              <button
                key={attr}
                className={`px-3 py-1 text-[10px] uppercase tracking-wide border transition-colors ${
                  idx === 0
                    ? 'bg-primary/20 border-primary text-primary'
                    : 'bg-[#2a2a2a] border-[#3a3a3a] text-muted-foreground hover:text-foreground'
                }`}
              >
                {attr}
              </button>
            ))}
          </div>

          {/* Spreadsheet */}
          <div className="flex-1 overflow-auto bg-background">
            <table className="w-full border-collapse text-[11px]">
              <thead className="sticky top-0 bg-[#1a1a1a] z-10">
                <tr>
                  <th className="border border-[#3a3a3a] bg-[#2a2a2a] px-3 py-2 text-left text-[10px] uppercase tracking-wide text-muted-foreground font-medium w-48">
                    Fixture
                  </th>
                  {columns.map((col) => (
                    <th
                      key={col}
                      className="border border-[#3a3a3a] bg-[#2a2a2a] px-3 py-2 text-center text-[10px] uppercase tracking-wide text-muted-foreground font-medium"
                    >
                      {col}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {fixtures.map((fixture, rowIdx) => (
                  <tr key={rowIdx}>
                    <td className="border border-[#3a3a3a] bg-[#1a1a1a] px-3 py-2 text-muted-foreground">
                      {fixture.name}
                    </td>
                    {columns.slice(0, -1).map((col, colIdx) => {
                      const key = col.toLowerCase().replace('lx', 'lx');
                      const value = fixture[key as keyof typeof fixture];
                      const hasValue = value && value !== '';
                      
                      return (
                        <td
                          key={colIdx}
                          className={`border border-[#3a3a3a] px-3 py-2 text-center ${
                            hasValue ? 'bg-[#3a3a2a] text-primary' : 'bg-[#1a1a1a]'
                          }`}
                        >
                          {value}
                        </td>
                      );
                    })}
                    <td className="border border-[#3a3a3a] bg-[#1a1a1a] px-3 py-2 text-center text-muted-foreground">
                      ...
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>

      {/* Footer Bar */}
      <div className="flex-shrink-0 bg-[#1a1a1a] border-t border-[#3a3a3a] px-4 py-2 flex items-center justify-between">
        <div className="flex items-center gap-6">
          <div className="flex items-center gap-2">
            <div className="w-1.5 h-1.5 rounded-full bg-primary" />
            <span className="text-[10px] uppercase tracking-wider text-foreground">Command Line</span>
          </div>
          <div className="flex-1 min-w-[400px]">
            <input
              type="text"
              placeholder="Enter command..."
              className="w-full bg-[#0a0a0a] border border-[#3a3a3a] px-3 py-1.5 text-xs text-foreground placeholder:text-muted-foreground focus:outline-none focus:border-primary/50"
            />
          </div>
        </div>
        <div className="flex items-center gap-6 text-[10px]">
          <span className="text-muted-foreground">DMX: 512/512</span>
          <span className="text-muted-foreground">Programmer: Empty</span>
          <span className="text-muted-foreground">Selection: 0 Fixtures</span>
          <div className="flex items-center gap-2">
            <span className="text-muted-foreground">Status:</span>
            <div className="w-2 h-2 rounded-full bg-[#4a9e5c]" />
            <span className="text-foreground">Ready</span>
          </div>
        </div>
      </div>
    </div>
  );
}
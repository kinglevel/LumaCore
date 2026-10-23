import React, { useRef, useEffect, useState } from 'react';
import { TabConfig } from '../../types/workspace';
import { Plus, X, MoreHorizontal, SplitSquareHorizontal, SplitSquareVertical, LayoutGrid } from 'lucide-react';

interface TabBarProps {
  tabs: TabConfig[];
  activeTabId: string;
  onTabChange: (tabId: string) => void;
  onTabClose: (tabId: string) => void;
  onNewTab: () => void;
  onSplitHorizontal?: () => void;
  onSplitVertical?: () => void;
  onClosePane?: () => void;
}

export function TabBar({ 
  tabs, 
  activeTabId, 
  onTabChange, 
  onTabClose, 
  onNewTab,
  onSplitHorizontal,
  onSplitVertical,
  onClosePane
}: TabBarProps) {
  const [overflowMenuOpen, setOverflowMenuOpen] = useState(false);
  const [splitMenuOpen, setSplitMenuOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const tabsContainerRef = useRef<HTMLDivElement>(null);
  const [tabWidth, setTabWidth] = useState<number | null>(null);

  const DEFAULT_TAB_WIDTH = 120; // Default tab width in pixels
  const MIN_TAB_WIDTH = 88; // Minimum tab width before creating overflow
  const CONTROLS_WIDTH = 160; // Space for + button, overflow menu, and controls

  useEffect(() => {
    const calculateTabWidth = () => {
      if (!containerRef.current || tabs.length === 0) return;
      
      const containerWidth = containerRef.current.offsetWidth;
      const availableWidth = containerWidth - CONTROLS_WIDTH;
      
      // Calculate width per tab
      const calculatedWidth = availableWidth / tabs.length;
      
      // If calculated width is less than minimum, use minimum (which will cause overflow)
      if (calculatedWidth < MIN_TAB_WIDTH) {
        setTabWidth(MIN_TAB_WIDTH);
      } else if (calculatedWidth > DEFAULT_TAB_WIDTH) {
        // Cap at default width
        setTabWidth(DEFAULT_TAB_WIDTH);
      } else {
        // Use calculated width (tabs will shrink)
        setTabWidth(calculatedWidth);
      }
    };

    calculateTabWidth();
    window.addEventListener('resize', calculateTabWidth);
    return () => window.removeEventListener('resize', calculateTabWidth);
  }, [tabs.length]);

  return (
    <div 
      ref={containerRef}
      className="flex items-center gap-0.5 border-b border-border bg-secondary/50"
    >
      <div className="flex items-center gap-0.5 flex-1 overflow-x-auto">
        {/* Visible Tabs */}
        <div ref={tabsContainerRef} className="flex items-center gap-0.5">
          {tabs.map((tab) => {
            const isActive = tab.id === activeTabId;
            return (
              <button
                key={tab.id}
                onClick={() => onTabChange(tab.id)}
                className={`
                  group relative px-3 py-2 text-xs transition-colors border-r border-border
                  flex items-center gap-2 min-w-[${MIN_TAB_WIDTH}px] flex-shrink-0
                  ${isActive 
                    ? 'bg-card text-foreground' 
                    : 'bg-secondary/30 text-muted-foreground hover:bg-secondary hover:text-foreground'
                  }
                `}
                style={{ width: tabWidth ? `${tabWidth}px` : 'auto' }}
              >
                <div className={`w-1.5 h-1.5 rounded-full transition-colors ${
                  isActive ? 'bg-primary' : 'bg-border'
                }`} />
                <span className="uppercase tracking-wide truncate flex-1">{tab.label}</span>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onTabClose(tab.id);
                  }}
                  className="hover:text-[#c93838] hover:bg-secondary/50 rounded-sm p-0.5 transition-all"
                >
                  <X className="w-3 h-3" />
                </button>
                {isActive && (
                  <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-primary" />
                )}
              </button>
            );
          })}
        </div>
      </div>

      {/* Right Controls - Always Visible */}
      <div className="flex items-center gap-0.5 flex-shrink-0">
        {/* New Tab Button */}
        <button
          onClick={onNewTab}
          title="New Tab"
          className="px-2.5 py-2 text-xs bg-secondary/30 hover:bg-primary/20 text-muted-foreground hover:text-primary border-l border-border transition-colors"
        >
          <Plus className="w-3.5 h-3.5" />
        </button>

        {/* Overflow Menu */}
        {tabs.length > 0 && (
          <div className="relative">
            <button
              onClick={() => setOverflowMenuOpen(!overflowMenuOpen)}
              title="All Tabs"
              className="px-3 py-2 text-xs bg-secondary/30 hover:bg-secondary text-muted-foreground hover:text-foreground border-l border-border transition-colors"
            >
              <MoreHorizontal className="w-4 h-4" />
            </button>
            {overflowMenuOpen && (
              <>
                <div 
                  className="fixed inset-0 z-10" 
                  onClick={() => setOverflowMenuOpen(false)} 
                />
                <div className="absolute top-full right-0 mt-1 bg-card border border-border min-w-[200px] z-20 max-h-[400px] overflow-y-auto">
                  {tabs.map((tab) => (
                    <button
                      key={tab.id}
                      onClick={() => {
                        onTabChange(tab.id);
                        setOverflowMenuOpen(false);
                      }}
                      className="w-full px-3 py-2 text-xs text-left hover:bg-secondary transition-colors flex items-center justify-between group"
                    >
                      <span className="text-foreground uppercase tracking-wide">{tab.label}</span>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          onTabClose(tab.id);
                          if (tabs.length === 1) setOverflowMenuOpen(false);
                        }}
                        className="opacity-0 group-hover:opacity-100 hover:text-[#c93838]"
                      >
                        <X className="w-3 h-3" />
                      </button>
                    </button>
                  ))}
                </div>
              </>
            )}
          </div>
        )}

        {/* Split Controls */}
        {(onSplitHorizontal || onSplitVertical) && (
          <div className="relative">
            <button
              onClick={() => setSplitMenuOpen(!splitMenuOpen)}
              title="Split Pane"
              className="px-2 py-2 text-xs bg-secondary/30 hover:bg-secondary text-muted-foreground hover:text-foreground border-l border-border transition-colors"
            >
              <LayoutGrid className="w-3.5 h-3.5" />
            </button>
            {splitMenuOpen && (
              <>
                <div 
                  className="fixed inset-0 z-10" 
                  onClick={() => setSplitMenuOpen(false)} 
                />
                <div className="absolute top-full right-0 mt-1 bg-card border border-border min-w-[180px] z-20">
                  {onSplitVertical && (
                    <button
                      onClick={() => {
                        onSplitVertical();
                        setSplitMenuOpen(false);
                      }}
                      className="w-full px-3 py-2.5 text-xs text-left hover:bg-secondary transition-colors flex items-center gap-3"
                    >
                      <SplitSquareVertical className="w-4 h-4 text-muted-foreground" />
                      <span className="text-foreground uppercase tracking-wide">Split Horizontal</span>
                    </button>
                  )}
                  {onSplitHorizontal && (
                    <button
                      onClick={() => {
                        onSplitHorizontal();
                        setSplitMenuOpen(false);
                      }}
                      className="w-full px-3 py-2.5 text-xs text-left hover:bg-secondary transition-colors flex items-center gap-3"
                    >
                      <SplitSquareHorizontal className="w-4 h-4 text-muted-foreground" />
                      <span className="text-foreground uppercase tracking-wide">Split Vertical</span>
                    </button>
                  )}
                </div>
              </>
            )}
          </div>
        )}
        
        {/* Close Pane Button */}
        {onClosePane && (
          <button
            onClick={onClosePane}
            title="Close Pane"
            className="px-3 py-2 text-xs bg-secondary/30 hover:bg-[#c93838]/20 text-muted-foreground hover:text-[#c93838] border-l border-border transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        )}
      </div>
    </div>
  );
}
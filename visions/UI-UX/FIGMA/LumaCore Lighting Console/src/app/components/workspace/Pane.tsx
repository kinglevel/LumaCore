import React, { useState } from "react";
import { Plus } from "lucide-react";
import { PaneConfig, TabConfig, ViewType } from "../../types/workspace";
import { TabBar } from "./TabBar";
import { ViewSelector } from "./ViewSelector";
import { ViewRenderer } from "./ViewRenderer";

interface PaneProps {
  config: PaneConfig;
  onUpdate: (config: PaneConfig) => void;
  onSplitHorizontal?: () => void;
  onSplitVertical?: () => void;
  onRemove?: () => void;
}

const VIEW_LABELS: Record<ViewType, { label: string; shortLabel: string; priority: number }> = {
  'patch': { label: 'Patch', shortLabel: 'Patch', priority: 1 },
  'live': { label: 'Live', shortLabel: 'Live', priority: 1 },
  'cues': { label: 'Cues', shortLabel: 'Cues', priority: 1 },
  'groups': { label: 'Groups', shortLabel: 'Grps', priority: 2 },
  'effects': { label: 'Effects', shortLabel: 'FX', priority: 2 },
  'presets': { label: 'Presets', shortLabel: 'Pres', priority: 2 },
  'output-monitor': { label: 'Output Monitor', shortLabel: 'Output', priority: 2 },
  'executors': { label: 'Executors', shortLabel: 'Exec', priority: 3 },
  'command': { label: 'Command', shortLabel: 'Cmd', priority: 3 },
};

export function Pane({ config, onUpdate, onSplitHorizontal, onSplitVertical, onRemove }: PaneProps) {
  const [showViewSelector, setShowViewSelector] = useState(false);

  const activeTab = config.tabs.find(t => t.id === config.activeTabId);

  const handleTabChange = (tabId: string) => {
    onUpdate({ ...config, activeTabId: tabId });
  };

  const handleTabClose = (tabId: string) => {
    const newTabs = config.tabs.filter(t => t.id !== tabId);
    
    if (newTabs.length === 0) {
      // If no tabs left, remove this pane
      if (onRemove) {
        onRemove();
      }
      return;
    }

    const newActiveTabId = config.activeTabId === tabId 
      ? newTabs[0].id 
      : config.activeTabId;

    onUpdate({
      ...config,
      tabs: newTabs,
      activeTabId: newActiveTabId,
    });
  };

  const handleNewTab = () => {
    setShowViewSelector(true);
  };

  const handleViewSelect = (type: ViewType) => {
    const labelInfo = VIEW_LABELS[type];
    const newTab: TabConfig = {
      id: `tab-${Date.now()}`,
      type,
      label: labelInfo.label,
      shortLabel: labelInfo.shortLabel,
      priority: labelInfo.priority,
    };

    onUpdate({
      ...config,
      tabs: [...config.tabs, newTab],
      activeTabId: newTab.id,
    });

    setShowViewSelector(false);
  };

  return (
    <div className="flex flex-col h-full border border-border bg-card">
      <TabBar
        tabs={config.tabs}
        activeTabId={config.activeTabId}
        onTabChange={handleTabChange}
        onTabClose={handleTabClose}
        onNewTab={handleNewTab}
        onSplitHorizontal={onSplitHorizontal}
        onSplitVertical={onSplitVertical}
        onClosePane={onRemove}
      />

      <div className="flex-1 overflow-auto bg-background">
        {activeTab ? (
          <ViewRenderer type={activeTab.type} />
        ) : (
          <div className="flex items-center justify-center h-full">
            <button
              onClick={handleNewTab}
              className="flex items-center justify-center w-24 h-24 border-2 border-dashed border-border hover:border-primary bg-card hover:bg-primary/5 transition-all group"
            >
              <Plus className="w-12 h-12 text-border group-hover:text-primary transition-colors" />
            </button>
          </div>
        )}
      </div>

      {showViewSelector && (
        <ViewSelector
          onSelect={handleViewSelect}
          onCancel={() => setShowViewSelector(false)}
        />
      )}
    </div>
  );
}
import React, { useState } from "react";
import { LayoutConfig, PaneConfig, SplitConfig, isPaneConfig, isSplitConfig, ViewType } from "../../types/workspace";
import { Pane } from "./Pane";
import { ViewSelector } from "./ViewSelector";
import { Plus } from "lucide-react";
import { Panel, PanelGroup, PanelResizeHandle } from "react-resizable-panels";

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

interface WindowManagerProps {
  className?: string;
}

export function WindowManager({ className = "" }: WindowManagerProps) {
  const [layout, setLayout] = useState<LayoutConfig>(null);

  const handleInitialPaneCreate = () => {
    const newPane: PaneConfig = {
      id: `pane-${Date.now()}`,
      tabs: [],
      activeTabId: '',
    };
    setLayout(newPane);
  };

  const updateLayoutAtPath = (path: number[], updater: (layout: LayoutConfig) => LayoutConfig): void => {
    setLayout(prevLayout => {
      if (!prevLayout) return prevLayout;
      return updateAtPath(prevLayout, path, updater);
    });
  };

  const updateAtPath = (layout: LayoutConfig, path: number[], updater: (layout: LayoutConfig) => LayoutConfig): LayoutConfig => {
    if (path.length === 0) {
      return updater(layout);
    }

    if (isSplitConfig(layout)) {
      const [index, ...rest] = path;
      return {
        ...layout,
        children: layout.children.map((child, i) => 
          i === index ? updateAtPath(child, rest, updater) : child
        ),
      };
    }

    return layout;
  };

  const removeAtPath = (layout: LayoutConfig, path: number[]): LayoutConfig => {
    if (path.length === 0) {
      return null;
    }

    if (path.length === 1 && isSplitConfig(layout)) {
      const remainingChildren = layout.children.filter((_, i) => i !== path[0]);
      if (remainingChildren.length === 0) return null;
      if (remainingChildren.length === 1) return remainingChildren[0];
      return { ...layout, children: remainingChildren };
    }

    if (isSplitConfig(layout)) {
      const [index, ...rest] = path;
      const updatedChild = removeAtPath(layout.children[index], rest);
      
      if (!updatedChild) {
        const remainingChildren = layout.children.filter((_, i) => i !== index);
        if (remainingChildren.length === 0) return null;
        if (remainingChildren.length === 1) return remainingChildren[0];
        return { ...layout, children: remainingChildren };
      }

      return {
        ...layout,
        children: layout.children.map((child, i) => i === index ? updatedChild : child),
      };
    }

    return layout;
  };

  const splitPane = (path: number[], direction: 'horizontal' | 'vertical') => {
    updateLayoutAtPath(path, (currentLayout) => {
      if (!isPaneConfig(currentLayout)) return currentLayout;

      const newPane: PaneConfig = {
        id: `pane-${Date.now()}`,
        tabs: [],
        activeTabId: '',
      };

      // Swap direction: 'horizontal' split means top/bottom (vertical layout in react-resizable-panels)
      // 'vertical' split means left/right (horizontal layout in react-resizable-panels)
      const panelDirection = direction === 'horizontal' ? 'vertical' : 'horizontal';

      const split: SplitConfig = {
        id: `split-${Date.now()}`,
        direction: panelDirection,
        sizes: [50, 50],
        children: [currentLayout, newPane],
      };

      return split;
    });
  };

  const renderLayout = (config: LayoutConfig, path: number[] = []): React.ReactNode => {
    if (!config) return null;

    if (isPaneConfig(config)) {
      return (
        <Pane
          key={config.id}
          config={config}
          onUpdate={(updated) => updateLayoutAtPath(path, () => updated)}
          onSplitHorizontal={() => splitPane(path, 'horizontal')}
          onSplitVertical={() => splitPane(path, 'vertical')}
          onRemove={() => setLayout(prev => removeAtPath(prev, path))}
        />
      );
    }

    if (isSplitConfig(config)) {
      return (
        <PanelGroup 
          key={config.id}
          direction={config.direction}
          className="h-full"
        >
          {config.children.map((child, index) => (
            <React.Fragment key={child.id}>
              <Panel defaultSize={config.sizes[index]} minSize={20}>
                <div className="h-full">
                  {renderLayout(child, [...path, index])}
                </div>
              </Panel>
              {index < config.children.length - 1 && (
                <PanelResizeHandle className={
                  config.direction === 'horizontal' 
                    ? "w-1 bg-border hover:bg-primary transition-colors data-[resize-handle-active]:bg-primary"
                    : "h-1 bg-border hover:bg-primary transition-colors data-[resize-handle-active]:bg-primary"
                } />
              )}
            </React.Fragment>
          ))}
        </PanelGroup>
      );
    }

    return null;
  };

  // Empty state
  if (!layout) {
    return (
      <div className={`flex items-center justify-center h-full bg-background ${className}`}>
        <button
          onClick={handleInitialPaneCreate}
          className="flex items-center justify-center w-24 h-24 border-2 border-dashed border-border hover:border-primary bg-card hover:bg-primary/5 transition-all group"
        >
          <Plus className="w-12 h-12 text-border group-hover:text-primary transition-colors" />
        </button>
      </div>
    );
  }

  return (
    <div className={`h-full ${className}`}>
      {renderLayout(layout)}
    </div>
  );
}
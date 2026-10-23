import React from "react";
import { motion } from "motion/react";

interface Tab {
  id: string;
  label: string;
}

interface ConsoleTabsProps {
  tabs: Tab[];
  activeTab: string;
  onChange: (tabId: string) => void;
  className?: string;
}

export function ConsoleTabs({ tabs, activeTab, onChange, className = "" }: ConsoleTabsProps) {
  return (
    <div className={`flex gap-0.5 border-b border-border bg-secondary/50 ${className}`}>
      {tabs.map((tab) => {
        const isActive = tab.id === activeTab;
        return (
          <button
            key={tab.id}
            onClick={() => onChange(tab.id)}
            className={`
              relative px-4 py-2 text-sm transition-colors
              border-r border-border last:border-r-0
              ${isActive 
                ? 'bg-card text-foreground' 
                : 'bg-secondary/30 text-muted-foreground hover:bg-secondary hover:text-foreground'
              }
            `}
          >
            {/* Amber indicator dot */}
            <div className="flex items-center gap-2">
              <div 
                className={`w-1.5 h-1.5 rounded-full transition-colors ${
                  isActive ? 'bg-primary' : 'bg-border'
                }`}
              />
              <span className="uppercase tracking-wide">{tab.label}</span>
            </div>
            
            {/* Active indicator line */}
            {isActive && (
              <motion.div
                layoutId="activeTab"
                className="absolute bottom-0 left-0 right-0 h-0.5 bg-primary"
                initial={false}
                transition={{ type: "spring", stiffness: 500, damping: 30 }}
              />
            )}
          </button>
        );
      })}
    </div>
  );
}
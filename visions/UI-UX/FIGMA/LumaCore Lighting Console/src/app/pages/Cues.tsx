import React from "react";
import { useState, useEffect } from "react";
import { Scene } from "../types/lighting";
import { Play, Pause, Clock, Layers, Plus, Trash2 } from "lucide-react";
import { motion } from "motion/react";

export default function Cues() {
  const [scenes, setScenes] = useState<Scene[]>([]);
  const [activeScene, setActiveScene] = useState<string | null>(null);

  useEffect(() => {
    const stored = localStorage.getItem("lightingScenes");
    if (stored) {
      const parsed = JSON.parse(stored);
      setScenes(parsed);
    }
  }, []);

  const handlePlayScene = (sceneId: string) => {
    setActiveScene(activeScene === sceneId ? null : sceneId);
  };

  const handleDeleteScene = (sceneId: string) => {
    const updated = scenes.filter(s => s.id !== sceneId);
    setScenes(updated);
    localStorage.setItem("lightingScenes", JSON.stringify(updated));
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleString("en-US", {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  if (scenes.length === 0) {
    return (
      <div className="flex flex-col h-full p-4">
        <div className="border border-border bg-card flex-1 flex flex-col">
          <div className="border-b border-border bg-secondary/30 px-4 py-2">
            <div className="flex items-center gap-2">
              <div className="w-1.5 h-1.5 rounded-full bg-primary" />
              <span className="text-xs uppercase tracking-wider text-foreground">Cue List</span>
            </div>
          </div>
          <div className="flex-1 flex items-center justify-center">
            <div className="text-center">
              <Layers className="w-12 h-12 text-muted-foreground mx-auto mb-4" />
              <h3 className="text-sm uppercase tracking-wide mb-2">No Cues Recorded</h3>
              <p className="text-xs text-muted-foreground">
                Navigate to Live control and record scenes to populate cue database
              </p>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full gap-4 p-4">
      {/* Stats Module */}
      <div className="border border-border bg-card flex-shrink-0">
        <div className="border-b border-border bg-secondary/30 px-4 py-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-1.5 h-1.5 rounded-full bg-primary" />
              <span className="text-xs uppercase tracking-wider text-foreground">Cue Stack Overview</span>
            </div>
            <button className="flex items-center gap-2 px-3 py-1 border border-border bg-card hover:bg-secondary transition-colors text-xs">
              <Plus className="w-3.5 h-3.5" />
              <span className="uppercase tracking-wide">New Cue</span>
            </button>
          </div>
        </div>
        <div className="grid grid-cols-4 divide-x divide-border">
          <div className="p-4 bg-secondary/10">
            <div className="text-2xl font-mono text-primary mb-1">{scenes.length}</div>
            <div className="text-xs text-muted-foreground uppercase tracking-wide">Total Cues</div>
          </div>
          <div className="p-4 bg-secondary/10">
            <div className="text-2xl font-mono text-primary mb-1">
              {activeScene ? '1' : '0'}
            </div>
            <div className="text-xs text-muted-foreground uppercase tracking-wide">Active</div>
          </div>
          <div className="p-4 bg-secondary/10">
            <div className="text-2xl font-mono text-primary mb-1">
              {scenes[0]?.fadeTime || 0}s
            </div>
            <div className="text-xs text-muted-foreground uppercase tracking-wide">Avg Fade</div>
          </div>
          <div className="p-4 bg-secondary/10">
            <div className="text-2xl font-mono text-primary mb-1">
              {scenes.reduce((sum, s) => sum + s.fixtureStates.length, 0)}
            </div>
            <div className="text-xs text-muted-foreground uppercase tracking-wide">Fixture States</div>
          </div>
        </div>
      </div>

      {/* Cue List */}
      <div className="border border-border bg-card flex-1 flex flex-col min-h-0">
        <div className="border-b border-border bg-secondary/30 px-4 py-2">
          <div className="flex items-center gap-2">
            <div className="w-1.5 h-1.5 rounded-full bg-primary" />
            <span className="text-xs uppercase tracking-wider text-foreground">Cue Sequence</span>
          </div>
        </div>

        {/* Table Header */}
        <div className="grid grid-cols-12 gap-4 px-4 py-2 border-b border-border bg-secondary/20 text-xs text-muted-foreground uppercase flex-shrink-0">
          <div className="col-span-1">Cue</div>
          <div className="col-span-3">Name</div>
          <div className="col-span-2">Fade</div>
          <div className="col-span-2">Fixtures</div>
          <div className="col-span-2">Created</div>
          <div className="col-span-2">Actions</div>
        </div>

        {/* Cue Rows */}
        <div className="divide-y divide-border flex-1 overflow-y-auto">
          {scenes.map((scene, index) => {
            const isActive = activeScene === scene.id;
            return (
              <motion.div
                key={scene.id}
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ delay: index * 0.02 }}
                className={`grid grid-cols-12 gap-4 px-4 py-3 transition-colors ${
                  isActive ? 'bg-primary/10' : 'bg-secondary/5 hover:bg-secondary/10'
                }`}
              >
                <div className="col-span-1 flex items-center">
                  <div className="flex items-center gap-2">
                    {isActive && (
                      <div className="w-1.5 h-1.5 rounded-full bg-[#4a9e5c] animate-pulse" />
                    )}
                    <span className="text-sm font-mono text-primary">
                      {scene.cueNumber.toString().padStart(3, '0')}
                    </span>
                  </div>
                </div>
                <div className="col-span-3 flex items-center">
                  <span className="text-sm text-foreground truncate">{scene.name}</span>
                </div>
                <div className="col-span-2 flex items-center">
                  <div className="flex items-center gap-1.5 text-xs">
                    <Clock className="w-3 h-3 text-muted-foreground" />
                    <span className="font-mono text-foreground">{scene.fadeTime}s</span>
                  </div>
                </div>
                <div className="col-span-2 flex items-center">
                  <div className="flex items-center gap-1.5 text-xs">
                    <Layers className="w-3 h-3 text-muted-foreground" />
                    <span className="font-mono text-foreground">{scene.fixtureStates.length}</span>
                  </div>
                </div>
                <div className="col-span-2 flex items-center">
                  <span className="text-xs text-muted-foreground">
                    {formatDate(scene.createdAt)}
                  </span>
                </div>
                <div className="col-span-2 flex items-center gap-2">
                  <button
                    onClick={() => handlePlayScene(scene.id)}
                    className={`px-3 py-1 border text-xs uppercase transition-colors ${
                      isActive
                        ? 'border-[#4a9e5c] bg-[#4a9e5c]/20 text-[#4a9e5c]'
                        : 'border-border bg-card hover:bg-secondary text-muted-foreground hover:text-foreground'
                    }`}
                  >
                    {isActive ? (
                      <>
                        <Pause className="w-3 h-3 inline mr-1" />
                        Stop
                      </>
                    ) : (
                      <>
                        <Play className="w-3 h-3 inline mr-1" />
                        Go
                      </>
                    )}
                  </button>
                  <button
                    onClick={() => handleDeleteScene(scene.id)}
                    className="p-1 border border-border bg-card hover:bg-secondary text-muted-foreground hover:text-[#c93838] transition-colors"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                  </button>
                </div>
              </motion.div>
            );
          })}
        </div>
      </div>

      {/* Playback Controls */}
      <div className="border border-border bg-card flex-shrink-0">
        <div className="border-b border-border bg-secondary/30 px-4 py-2">
          <div className="flex items-center gap-2">
            <div className="w-1.5 h-1.5 rounded-full bg-primary" />
            <span className="text-xs uppercase tracking-wider text-foreground">Playback Transport</span>
          </div>
        </div>
        <div className="p-4 bg-secondary/10">
          <div className="flex items-center justify-center gap-3">
            <button className="px-6 py-3 border border-border bg-card hover:bg-secondary text-foreground text-xs uppercase tracking-wider transition-colors">
              Previous
            </button>
            <button className="px-8 py-3 bg-primary hover:bg-primary/90 text-primary-foreground text-xs uppercase tracking-wider transition-colors">
              <Play className="w-4 h-4 inline mr-2" />
              GO
            </button>
            <button className="px-6 py-3 border border-border bg-card hover:bg-secondary text-foreground text-xs uppercase tracking-wider transition-colors">
              Next
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
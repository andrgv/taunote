import React, { useState, useEffect } from "react";
import { v4 as uuid } from "uuid";
import Sidebar from "./components/Sidebar";
import WelcomeView from "./components/WelcomeView";
import ProjectView from "./components/ProjectView";
import { invoke } from "@tauri-apps/api/core";
import {
  AudioProject as DBAudioProject,
  ProjectGroup as DBProjectGroup,
  AppView,
} from "./types";

// UI-facing types for Sidebar and views
type UIAudioProject = {
  id: string;
  name: string;
  date: string;
  type: "meeting" | "lecture" | "other";
};

type UIProjectGroup = {
  id: string;
  name: string;
  audioProjects: UIAudioProject[];
};

function App() {
  const [projectGroups, setProjectGroups] = useState<UIProjectGroup[]>([]);
  const [selectedGroupId, setSelectedGroupId] = useState<string | null>(null);
  const [selectedAudioProjectId, setSelectedAudioProjectId] = useState<
    string | null
  >(null);
  const [currentView, setCurrentView] = useState<AppView>("welcome");

  // Load existing project groups from DB and map to UI types
  useEffect(() => {
    (async () => {
      try {
        await invoke("setup_backend");
        const groups = await invoke<DBProjectGroup[]>("get_project_groups");
        const uiGroups = groups.map((g) => ({
          id: g.id,
          name: g.name,
          audioProjects: g.audioProjects.map((ap) => ({
            id: ap.id,
            name: ap.name,
            date: ap.date,
            type: ap.project_type as "meeting" | "lecture" | "other",
          })),
        }));
        setProjectGroups(uiGroups);
      } catch (err) {
        console.error("Failed to init/load groups:", err);
      }
    })();
  }, []);

  // Create a new project group (in-memory)
  const handleNewProjectGroup = async () => {
    const newGroup: UIProjectGroup = {
      id: uuid(),
      name: `New Group ${projectGroups.length + 1}`,
      audioProjects: [],
    };

    try {
      await invoke("insert_project_group_to_db", {
        id: newGroup.id,
        name: newGroup.name,
      });
    } catch (e) {
      console.error("DB insert failed:", e);
      return;
    }
    setProjectGroups((prev) => [...prev, newGroup]);
    setSelectedGroupId(newGroup.id);
    setSelectedAudioProjectId(null);
    setCurrentView("welcome");
  };

  // Add a new audio project: update UI and persist to DB
  const handleNewAudioProject = async (groupId: string) => {
    const nextIdx =
      (projectGroups.find((g) => g.id === groupId)?.audioProjects.length ?? 0) +
      1;
    const uiAudio: UIAudioProject = {
      id: uuid(),
      name: `New Audio ${nextIdx}`,
      date: new Date().toISOString().slice(0, 10),
      type: "meeting",
    };

    // Optimistically update UI
    setProjectGroups((prev) =>
      prev.map((g) =>
        g.id === groupId
          ? { ...g, audioProjects: [...g.audioProjects, uiAudio] }
          : g,
      ),
    );
    setSelectedGroupId(groupId);
    setSelectedAudioProjectId(uiAudio.id);
    setCurrentView("project");

    // Persist to SQLite via Rust command
    const dbAudio: DBAudioProject = {
      id: uiAudio.id,
      group_id: groupId,
      name: uiAudio.name,
      relative_path: `projects/${groupId}/${uiAudio.id}`,
      date: uiAudio.date,
      project_type: uiAudio.type,
      language: "en",
    };
    try {
      await invoke("insert_audio_project_to_db", { audioProject: dbAudio });
    } catch (e) {
      console.error("DB insert failed:", e);
      // Optionally roll back UI change
    }
  };

  // Select an existing audio project
  const handleSelectAudioProject = (
    groupId: string,
    audioProjectId: string,
  ) => {
    setSelectedGroupId(groupId);
    setSelectedAudioProjectId(audioProjectId);
    setCurrentView("project");
  };

  // Handle file upload by sending it through the backend pipeline
  const handleFileUploaded = async (
    filePath: string,
    lang: string,
    type: "meeting" | "lecture" | "other",
  ) => {
    // Make sure a group exists to put this audio in
    let groupId = selectedGroupId;
    if (!groupId) {
      // Auto-create a group if none selected
      const newGroup = { id: uuid(), name: "Imported", audioProjects: [] };
      await invoke("insert_project_group_to_db", {
        id: newGroup.id,
        name: newGroup.name,
      });
      setProjectGroups((prev) => [...prev, newGroup]);
      groupId = newGroup.id;
    }

    // Create UI + DB audio project entry
    const uiAudio = {
      id: uuid(),
      name: filePath.split(/[/\\]/).pop() || "Untitled",
      date: new Date().toISOString().slice(0, 10),
      type,
    };

    setProjectGroups((prev) =>
      prev.map((g) =>
        g.id === groupId
          ? { ...g, audioProjects: [...g.audioProjects, uiAudio] }
          : g,
      ),
    );

    // Persist
    const dbAudio: DBAudioProject = {
      id: uiAudio.id,
      group_id: groupId,
      name: uiAudio.name,
      relative_path: filePath,
      date: uiAudio.date,
      project_type: type,
      language: lang,
    };
    await invoke("insert_audio_project_to_db", { audioProject: dbAudio });

    // Transcribe
    const transcriptPath = await invoke<string>("transcribe_audio", {
      audioPath: filePath,
      lang,
    });

    console.log("Transcript at:", transcriptPath);

    // Switch view
    setSelectedGroupId(groupId);
    setSelectedAudioProjectId(uiAudio.id);
    setCurrentView("project");
  };

  // Prepare props for ProjectView
  const selected = (() => {
    const grp = projectGroups.find((g) => g.id === selectedGroupId);
    const aud = grp?.audioProjects.find(
      (ap) => ap.id === selectedAudioProjectId,
    );
    return aud && grp
      ? {
          groupId: grp.id,
          groupName: grp.name,
          audioProjectId: aud.id,
          audioProjectName: aud.name,
        }
      : null;
  })();

  return (
    <div className="flex h-screen bg-background text-gray-100 font-inter">
      <Sidebar
        projectGroups={projectGroups}
        onNewProjectGroup={handleNewProjectGroup}
        onNewAudioProject={handleNewAudioProject}
        onSelectAudioProject={handleSelectAudioProject}
        selectedGroupId={selectedGroupId}
        selectedAudioProjectId={selectedAudioProjectId}
      />

      {currentView === "project" && selected && (
        <ProjectView
          groupId={selected.groupId}
          groupName={selected.groupName}
          audioProjectId={selected.audioProjectId}
          audioProjectName={selected.audioProjectName}
          onBack={() => setCurrentView("welcome")}
        />
      )}
      {currentView === "welcome" && (
        <WelcomeView onFileUploaded={handleFileUploaded} />
      )}
    </div>
  );
}

export default App;

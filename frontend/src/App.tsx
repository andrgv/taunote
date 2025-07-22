import React, { useState, useEffect } from 'react';
import { v4 as uuid } from 'uuid';
import Sidebar from './components/Sidebar';
import WelcomeView from './components/WelcomeView';
import ProjectView from './components/ProjectView';

// Interfaces (can be moved to a shared types file like frontend/src/types.ts)
interface AudioProject {
  id: string;
  name: string;
  date: string;
  type: 'meeting' | 'lecture' | 'other';
}

interface ProjectGroup {
  id: string;
  name: string;
  audioProjects: AudioProject[];
}

type AppView = 'welcome' | 'project';

function App() {
  const [projectGroups, setProjectGroups] = useState<ProjectGroup[]>([]);
  const [selectedGroupId, setSelectedGroupId] = useState<string | null>(null);
  const [selectedAudioProjectId, setSelectedAudioProjectId] = useState<string | null>(null);
  const [currentView, setCurrentView] = useState<AppView>('welcome');

  // TODO: change dummy values
  useEffect(() => {
    const initialGroups: ProjectGroup[] = [
      {
        id: uuid(),
        name: 'Q4 Planning',
        audioProjects: [
          { id: uuid(), name: 'Team Standup', date: '2024-10-15', type: 'meeting' },
          { id: uuid(), name: 'Product Review', date: '2024-10-14', type: 'meeting' },
        ],
      },
      {
        id: uuid(),
        name: 'Training Sessions',
        audioProjects: [
          { id: uuid(), name: 'React Fundamentals', date: '2024-10-12', type: 'lecture' },
        ],
      },
    ];
    setProjectGroups(initialGroups);
  }, []);

  // Handler to create a new overarching project group
  const handleNewProjectGroup = () => {
    const newGroup: ProjectGroup = {
      id: uuid(),
      name: `New Group ${projectGroups.length + 1}`,
      audioProjects: [], // New groups start with no audio projects
    };
    setProjectGroups((prevGroups) => [...prevGroups, newGroup]);
    // Optionally select the new group or keep current view
    setSelectedGroupId(newGroup.id);
    setSelectedAudioProjectId(null); // No audio project selected in the new group yet
    setCurrentView('welcome');
  };

  // Handler to add a new audio project to a specific group
  const handleNewAudioProject = (groupId: string) => {
    setProjectGroups((prevGroups) => {
      return prevGroups.map((group) => {
        if (group.id === groupId) {
          const newAudio: AudioProject = {
            id: uuid(),
            name: `New Audio ${group.audioProjects.length + 1}`,
            date: new Date().toISOString().slice(0, 10),
            type: 'meeting', // Default type for new audio
          };
          // Add the new audio project to this group
          const updatedAudioProjects = [...group.audioProjects, newAudio];
          // Set the newly created audio project as selected
          setSelectedGroupId(groupId);
          setSelectedAudioProjectId(newAudio.id);
          setCurrentView('welcome');
          return { ...group, audioProjects: updatedAudioProjects };
        }
        return group;
      });
    });
  };

  // Handler to select an existing audio project
  const handleSelectAudioProject = (groupId: string, audioProjectId: string) => {
    setSelectedGroupId(groupId);
    setSelectedAudioProjectId(audioProjectId);
    // TODO: change
    setCurrentView('welcome'); // For now, selecting an audio project just shows welcome view
  };

  // Helper to get the name of the currently selected audio project and its group
  const getSelectedAudioProjectDetails = () => {
    const group = projectGroups.find(g => g.id === selectedGroupId);
    if (group) {
      const audioProject = group.audioProjects.find(ap => ap.id === selectedAudioProjectId);
      if (audioProject) {
        return {
          groupId: group.id,
          groupName: group.name,
          audioProjectId: audioProject.id,
          audioProjectName: audioProject.name,
        };
      }
    }
    return null;
  };

  const selectedAudioProjectDetails = getSelectedAudioProjectDetails();

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

      {/* Conditional rendering of the main content area */}
      {currentView === 'welcome' && <WelcomeView />}
      {currentView === 'welcome' && selectedAudioProjectDetails && (
        <NewAudioView
          groupId={selectedAudioProjectDetails.groupId}
          groupName={selectedAudioProjectDetails.groupName}
          audioProjectId={selectedAudioProjectDetails.audioProjectId}
          audioProjectName={selectedAudioProjectDetails.audioProjectName}
        />
      )}
    </div>
  );
}

export default App;

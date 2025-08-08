import React from "react";
import { Menu, Plus, Settings, Calendar, BookOpen } from "lucide-react";

// TODO: move Interfacesto somewhere better frontend/src/types.ts???
interface AudioProject {
  id: string;
  name: string;
  date: string;
  type: "meeting" | "lecture" | "other";
}

interface ProjectGroup {
  id: string;
  name: string;
  audioProjects: AudioProject[];
}

interface SidebarProps {
  projectGroups: ProjectGroup[];
  onNewProjectGroup: () => void;
  onNewAudioProject: (groupId: string) => void;
  onSelectAudioProject: (groupId: string, audioProjectId: string) => void;
  selectedGroupId: string | null;
  selectedAudioProjectId: string | null;
}

const Sidebar = ({
  projectGroups,
  onNewProjectGroup,
  onNewAudioProject,
  onSelectAudioProject,
  selectedGroupId,
  selectedAudioProjectId,
}: SidebarProps) => {
  return (
    <aside className="w-64 bg-sidebar p-4 flex flex-col justify-between rounded-r-lg shadow-lg h-screen">
      <div className="flex flex-col flex-1 overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between mb-8">
          <h1 className="text-xl font-bold flex items-center">
            <span className="bg-mint text-sidebar w-10 h-10 flex items-center justify-center rounded-full mr-2">
              τ
            </span>
            Taunote
          </h1>
          <button className="p-2 rounded-md hover:bg-inputBg">
            <Menu size={20} />
          </button>
        </div>

        {/* New Project Group Button */}
        <button
          onClick={onNewProjectGroup}
          className="w-full bg-accentPurple hover:bg-opacity-80 text-white py-2 px-4 rounded-lg flex items-center justify-center mb-6 shadow-md transition-colors duration-200"
        >
          <Plus size={20} className="mr-2" />
          New Project Group
        </button>

        {/* Project Groups List */}
        <div className="flex-1 overflow-y-auto pr-1">
          <nav className="space-y-4">
            {projectGroups.map((group) => (
              <div key={group.id}>
                <h2 className="text-sm font-semibold text-gray-400 mb-2 flex items-center justify-between">
                  {group.name}
                  <button
                    onClick={() => onNewAudioProject(group.id)} // Add new audio to this group
                    className="ml-2 p-1 rounded-full bg-gray-700 hover:bg-gray-600 transition-colors duration-200"
                    title={`Add new audio to ${group.name}`}
                  >
                    <Plus size={14} className="text-gray-300" />
                  </button>
                </h2>
                <ul className="space-y-2">
                  {group.audioProjects.map((audioProject) => (
                    <li
                      key={audioProject.id}
                      className={`flex items-center p-2 rounded-md cursor-pointer transition-colors duration-200 ${
                        selectedGroupId === group.id &&
                        selectedAudioProjectId === audioProject.id
                          ? "bg-inputBg"
                          : "hover:bg-inputBg"
                      }`}
                      onClick={() =>
                        onSelectAudioProject(group.id, audioProject.id)
                      }
                    >
                      {audioProject.type === "meeting" ? (
                        <Calendar size={16} className="mr-2 text-gray-400" />
                      ) : (
                        <BookOpen size={16} className="mr-2 text-gray-400" />
                      )}
                      <span className="flex-1 truncate">
                        {audioProject.name} - {audioProject.date}
                      </span>
                      <span
                        className={`ml-2 px-2 py-1 rounded-full text-xs font-medium ${
                          audioProject.type === "meeting"
                            ? "bg-tagMeeting text-white"
                            : "bg-tagLecture text-white"
                        }`}
                      >
                        {audioProject.type}
                      </span>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </nav>
        </div>
      </div>

      {/* Settings Link */}
      <div className="border-t border-gray-700 pt-4">
        <a
          href="#"
          className="flex items-center p-2 rounded-md hover:bg-inputBg text-gray-300 transition-colors duration-200"
        >
          <Settings size={20} className="mr-2" />
          Settings
        </a>
      </div>
    </aside>
  );
};

export default Sidebar;

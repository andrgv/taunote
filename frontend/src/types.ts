export interface AudioProject {
  id: string;
  group_id: string;
  name: string;
  relative_path: string;
  date: string;
  project_type: string;
  language: string;
}

export interface ProjectGroup {
  id: string;
  name: string;
  audioProjects: AudioProject[];
}

export type AppView = "welcome" | "project";

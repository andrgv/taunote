import React, { useState, ChangeEvent } from 'react';
import Topbar from './Topbar';
// import Rightbar from './Rightbar';

interface ProjectViewProps {
    groupId: string;
    groupName: string;
    audioProjectId: string;
    audioProjectName: string;
    onBack: () => void;
}

// TODO: this should not be hardcoded here
const recordingTypes = ['meeting', 'lecture', 'other'] as const;
type RecordingType = typeof recordingTypes[number];

const languages = ['Auto', 'English', 'Spanish'] as const;
type Language = typeof languages[number];

const ProjectView = ({
    groupId,
    groupName,
    audioProjectId,
    audioProjectName,
    onBack
}: ProjectViewProps) => {
    const [recordingType, setRecordingType] = useState<RecordingType>('meeting');
    const [language, setLanguage] = useState<Language>('Auto');
    const [file, setFile] = useState<File | null>(null);

    const handleFileChange = (e: ChangeEvent<HTMLInputElement>) => {
        const chosen = e.target.files?.[0] ?? null;
        setFile(chosen);
    };
    return (
        <div className='flex flex-col h-screen'>
            <Topbar title={audioProjectName}>
                <button onClick={onBack} className="bg-gray-700 hover:bg-gray-600 text-white text-sm px-3 py-1 rounded">
                    Back
                </button>
            </Topbar>

            <div className='p-6 flex-1 overflow-y-auto'>
                <p className="text-gray-400">Group: {groupName}</p>
                <p className="text-gray-400">Audio Project ID: {audioProjectId}</p>
            </div>

            <div className='space-y-4 p-6 overflow-y-auto'>
                <div>
                    <label className="block text-sm font-medium mb-1">Recording Type</label>
                    <select className="bg-gray-800 border border-gray-700 p-2 rounded w-full" 
                            value={recordingType} 
                            onChange={(e) => setRecordingType(e.target.value as RecordingType)}
                    >
                        {recordingTypes.map((type) => (<option key={type} value={type}>{type}</option>))}
                    </select>
                </div>

                <div>
                    <label className="block text-sm font-medium mb-1">Language</label>
                    <select className="bg-gray-800 border border-gray-700 p-2 rounded w-full" 
                            value={language} 
                            onChange={(e) => setLanguage(e.target.value as Language)}
                    >
                        {languages.map((lang) => (<option key={lang} value={lang}>{lang}</option>))}
                    </select>
                </div>
            </div>
        </div>
    )
};
export default ProjectView;
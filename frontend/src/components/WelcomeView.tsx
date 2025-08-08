import React, { useState } from "react";
import { UploadCloud } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";

interface WelcomeViewProps {
  onFileUploaded: (
    filePath: string,
    lang: string,
    type: "meeting" | "lecture" | "other",
  ) => Promise<void>;
}

const WelcomeView: React.FC<WelcomeViewProps> = ({ onFileUploaded }) => {
  const [recordingType, setRecordingType] = useState<
    "Meeting" | "Lecture" | "Other"
  >("Meeting");
  const [language, setLanguage] = useState<
    "Auto" | "English" | "Spanish" | "French"
  >("Auto");

  const handlePick = async () => {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Audio", extensions: ["mp3", "wav", "m4a"] }],
    });
    if (!selected || Array.isArray(selected)) return;

    const langCode =
      language === "Auto"
        ? "auto"
        : language === "English"
          ? "en"
          : language === "Spanish"
            ? "es"
            : language === "French"
              ? "fr"
              : "auto";

    await onFileUploaded(
      selected,
      langCode,
      recordingType.toLowerCase() as any,
    );
  };

  return (
    <div className="flex-1 p-8 flex flex-col items-center justify-center">
      {/* Header */}
      <div className="text-center mb-10">
        <div className="text-5xl mb-4">
          <span className="bg-accentGreen text-sidebar w-20 h-20 flex items-center justify-center rounded-full">
            τ
          </span>
        </div>
        <h1 className="text-4xl font-bold mb-4">Welcome to Taunote</h1>
        <p className="text-lg text-gray-400 max-w-xl">
          Upload your audio recordings to get started with transcription and
          AI-powered summaries
        </p>
      </div>

      {/* Dropdowns */}
      <div className="flex space-x-4 mb-8">
        <div className="flex items-center">
          <label htmlFor="recording-type" className="mr-2 text-gray-400">
            Recording Type
          </label>
          <select
            id="recording-type"
            className="bg-inputBg text-white p-2 rounded-md"
            value={recordingType}
            onChange={(e) => setRecordingType(e.target.value as any)}
          >
            <option>Meeting</option>
            <option>Lecture</option>
            <option>Other</option>
          </select>
        </div>
        <div className="flex items-center">
          <label htmlFor="language" className="mr-2 text-gray-400">
            Language
          </label>
          <select
            id="language"
            className="bg-inputBg text-white p-2 rounded-md"
            value={language}
            onChange={(e) => setLanguage(e.target.value as any)}
          >
            <option>Auto</option>
            <option>English</option>
            <option>Spanish</option>
            <option>French</option>
          </select>
        </div>
      </div>

      {/* Drop/File Select Area */}
      <div
        className="w-full max-w-2xl h-64 border-2 border-dashed border-gray-600 rounded-xl flex flex-col items-center justify-center p-6 bg-inputBg hover:border-accentGreen transition-colors cursor-pointer"
        onClick={handlePick}
      >
        <UploadCloud size={48} className="text-gray-500 mb-4" />
        <p className="text-gray-400 text-lg mb-2">
          <span className="text-white font-semibold">
            Drop your audio file here
          </span>{" "}
          or click to browse
        </p>
        <p className="text-gray-500 text-sm">
          Supports MP3, WAV, M4A files up to 500MB
        </p>
      </div>
    </div>
  );
};

export default WelcomeView;

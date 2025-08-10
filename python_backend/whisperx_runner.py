import whisperx
import gc
import torch
from dotenv import load_dotenv
import os
from pathlib import Path
import argparse

load_dotenv()
hf_token = os.getenv("HUGGINGFACE_TOKEN")
if not hf_token:
    raise EnvironmentError("Missing HUGGINGFACE_TOKEN in .env")

def whisperx_runner(
        audio_file: str,
        output_path: str = "../tmp/transcript.txt",
        device: str = "cuda" if torch.cuda.is_available() else ("mps" if torch.backends.mps.is_available() else "cpu"),
        batch_size: int = 8, # reduce if low on GPU mem
        compute_type: str = None,
        language = None
):
        try:
                compute_type = compute_type or ("float16" if device == "cuda" else "int8")
                # Transcribe with original whisper (batched)
                print("[INFO] Loading STT model and audio...")
                model_dir = os.path.join('../models/whisperx/')
                model = whisperx.load_model("small", device, compute_type=compute_type, download_root=model_dir)

                audio = whisperx.load_audio(audio_file)
                print(f"[INFO] Transcribing: {audio_file}...")
                result = model.transcribe(audio, batch_size=batch_size, language=language) if language else model.transcribe(audio, batch_size=batch_size)
                print(result["segments"]) # before alignment
                print(f"[INFO] Transcription complete. Language: {result['language']}")

                # delete model if low on GPU resources
                cleanup_model(model, device)

                # Align whisper output
                print("[INFO] Loading alignment model...")
                model_a, metadata = whisperx.load_align_model(language_code=result["language"], device=device)
                result = whisperx.align(result["segments"], model_a, metadata, audio, device, return_char_alignments=False)

                print(result["segments"]) # after alignment

                # delete model if low on GPU resources
                cleanup_model(model_a, device)

                # Assign speaker labels
                print("[INFO] Performing speaker diarization...")
                diarize_model = whisperx.diarize.DiarizationPipeline(use_auth_token=hf_token, device=device)

                # add min/max number of speakers if known
                diarize_segments = diarize_model(audio)
                # diarize_model(audio, min_speakers=min_speakers, max_speakers=max_speakers)

                result = whisperx.assign_word_speakers(diarize_segments, result)
                print("[INFO] Transcription with speaker labels complete.")

                # save to output path
                Path(output_path).parent.mkdir(parents=True, exist_ok=True)
                with open(output_path, "w", encoding="utf-8") as f:
                        for segment in result["segments"]:
                                speaker = segment.get("speaker", "unknown")
                                text = segment["text"].strip()
                                f.write(f"[{speaker}] {text}\n")
                
                print(f"[INFO] Saved transcript to {output_path}")
                return result

        except Exception as e:
                print(f"[ERROR] WhisperX processing failed: {e}")
                return None


def cleanup_model(model, device):
        del model
        gc.collect()
        if device == "cuda":
                torch.cuda.empty_cache()


if __name__ == '__main__':
        parser = argparse.ArgumentParser(prog='whisperx_runner', description="Performs diariazation, transcription and timestamping on a recording")
        parser.add_argument("-i", "--input", required=True, help="Path to the audio file input")
        parser.add_argument("-o", "--output", required=False, help="Set custom output path for the transcript")
        parser.add_argument("-m", "--model", required=False, help="Set a custom whisper model")
        parser.add_argument("-l", "--lang", required=False, help="Set a language for the audio")

        args = parser.parse_args()
        whisperx_runner(
                audio_file=args.input,
                output_path=args.output or "tmp/transcript.txt",
                language=args.lang or None
        )
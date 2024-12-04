#!/usr/bin/env python3

import logging
import platform
import re
import subprocess
import sys
import threading
import time
from argparse import ArgumentParser
from pathlib import Path

import torch
from ctranslate2 import get_supported_compute_types
from faster_whisper import (BatchedInferencePipeline, WhisperModel,
                            format_timestamp)

supported_compute_types = list(get_supported_compute_types(
    "cuda" if torch.cuda.is_available() else "cpu"))

# Argument parser for command-line options
stdin_parser = ArgumentParser(description="Transcribe video file")
stdin_parser.add_argument(
    "-c",
    metavar="compute_type",
    help=f"Compute type for whisper. Choices: {
        supported_compute_types}",
    default="default",
    choices=supported_compute_types
)
stdin_parser.add_argument(
    "-l",
    metavar="language",
    help="Set language for whisper",
    default=None
)
stdin_parser.add_argument(
    "-f",
    metavar="file",
    help="file input",
    required=True,
    nargs='+',
    type=Path
)

ARGS = stdin_parser.parse_args()
MODEL_NAME = "mlx-community/whisper-large-v3-mlx" if platform.system(
) == "Darwin" else "large-v3"   # Model size based on hardware
ALLOWED_EXTENSIONS = [".mp4", ".mp3", ".mov", ".mkv", ".webm"]
EXCLUDE_DIRS = [
    "_NICHT SENDEN",
    "00-assets",
    "00-audio",
    "00-social-share",
    "Black Error",
    "03 - Musikalische Lückenfüller"
]
LOCK_EXT = ".lock"


class Logger:
    def __init__(self, name: str):
        self.logger = logging.getLogger(name)
        self.logger.setLevel(logging.DEBUG)

        sh = logging.StreamHandler(sys.stderr)
        sh.setLevel(logging.DEBUG)

        formatter = logging.Formatter("[%(levelname)s] %(message)s")
        sh.setFormatter(formatter)

        self.logger.addHandler(sh)

    def debug(self, msg: str):
        self.logger.debug(msg)

    def info(self, msg: str):
        self.logger.info(msg)

    def warning(self, msg: str):
        self.logger.warning(msg)

    def error(self, msg: str):
        self.logger.error(msg)

    def critical(self, msg: str):
        self.logger.critical(msg)
        exit(1)


# Function to check if a video is already being processed
def is_locked(video_path: Path):
    return video_path.with_suffix(LOCK_EXT).is_file()


# Function to check if a video is already transcribed
def is_transcribed(video_path: Path):
    return video_path.with_suffix('.vtt').is_file()


# Function to create a lock file for the video being processed
def lock_video(video_path: Path):
    with open(video_path.with_suffix(LOCK_EXT), "w") as lock_file:
        lock_file.write(str(time.time()))


# Function to remove the lock file after processing
def unlock_video(video_path: Path):
    video_path.with_suffix(LOCK_EXT).unlink(missing_ok=True)


def transcribe_video(video_path: Path):
    lang = ARGS.l

    if lang == "Auto" or lang == "auto":
        lang = None

    # Load Faster Whisper model (e.g., use GPU if available)
    model = WhisperModel(
        MODEL_NAME,
        device="cuda" if torch.cuda.is_available() else "cpu",
        compute_type=ARGS.c,
        download_root=Path.home().joinpath('.cache/huggingface/hub')
    )
    batched_model = BatchedInferencePipeline(model=model)

    try:
        segments, info = batched_model.transcribe(
            video_path, vad_filter=True, language=lang)
    except Exception as e:
        log.error(f"Failed to transcribe {video_path}: {e}")
        return

    log.info(f"Processing {video_path}")

    lock_video(video_path)
    vtt_path = video_path.with_suffix('.vtt')
    total_duration = info.duration
    progress = 0

    try:
        with open(vtt_path, "w", encoding="utf-8") as vtt_file:
            vtt_file.write("WEBVTT\n\n")

            for segment in segments:
                start_time = format_timestamp(segment.start)
                end_time = format_timestamp(segment.end)
                vtt_file.write(f"\n{start_time} --> {end_time}\n")
                vtt_file.write(f"{segment.text.strip()}\n")

                progress += (segment.end - segment.start)
                percent_complete = int((progress / total_duration) * 100)

                print(percent_complete)

        log.info(f"Transcription completed for {
                 video_path}, saved to {vtt_path}")
    except KeyboardInterrupt:
        vtt_path.unlink(missing_ok=True)
        log.warning(f"Transcription interrupted, cleanup: {video_path}")
    finally:
        unlock_video(video_path)
        print(100)


def read_stream(stream, callback):
    for line in iter(stream.readline, ''):
        callback(line.strip())
    stream.close()


def process_output(line):
    result = re.search(r"^(\d+)%", line.strip())
    if result is not None:
        g = result.group(1)
        print(g)


def transcribe_video_mlx(video_path: Path):
    lang = ARGS.l

    if lang == "Auto" or lang == "auto":
        lang = None

    log.info(f"Processing {video_path}")

    lock_video(video_path)
    vtt_path = video_path.with_suffix('.vtt')

    # this is some hacky workaround as mlx_whisper doesn't support iterable segments to update the progress.
    # so we just run mlx inside a thread and read out the percentage from stderr with regex.
    try:
        python_code = (
            f"from mlx_whisper import transcribe, writers; "
            f"result = transcribe(audio=r'{video_path}', "
            f"path_or_hf_repo=r'{MODEL_NAME}', language={
                repr(lang)}, verbose=False, "
            f"condition_on_previous_text=False); "
            f"writer_vtt = writers.get_writer(output_format='vtt', output_dir=r'{
                video_path.parent}'); "
            f"writer_vtt(result, '{vtt_path}')"
        )
        command = ["python", "-c", python_code]
        log.debug(" ".join(command))

        process = subprocess.Popen(
            command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)

        stderr_thread = threading.Thread(
            target=read_stream, args=(process.stderr, process_output))
        stderr_thread.start()
        stderr_thread.join()
        process.wait()

        vtt_path = video_path.with_suffix('.vtt')
        log.info(f"Transcription completed for {
            video_path}, saved to {vtt_path}")
    except Exception as e:
        log.error(f"Failed to transcribe {video_path}: {e}")
        return
    finally:
        unlock_video(video_path)
        print(100)


# Main function
if __name__ == "__main__":
    log = Logger(__name__)

    if platform.system() == "Darwin":
        log.info("Whisper will run with MLX support")
    elif torch.cuda.is_available():
        log.info("Whisper will run with CUDA on GPU")
    else:
        log.warning("Whisper will run on CPU")

    for f in ARGS.f:
        if f.is_file():
            if platform.system() == "Darwin":
                transcribe_video_mlx(f)
            else:
                transcribe_video(f)
        elif f.is_dir():
            for p in f.rglob('*'):
                if not any(ex_dir in str(p) for ex_dir in EXCLUDE_DIRS) and \
                    p.suffix in ALLOWED_EXTENSIONS and \
                        not is_locked(p) and not is_transcribed(p):
                    if platform.system() == "Darwin":
                        transcribe_video_mlx(p)
                    else:
                        transcribe_video(p)

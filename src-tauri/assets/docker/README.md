## Build image:

```bash
podman build -t faster_whisper -f docker/Dockerfile .
```

## Run command:

```bash
docker run -it --name whisper2 --gpus 0 -v /mnt/playout/tv-media:/tv-media -v ~/.cache/huggingface/hub:/root/.cache/huggingface/hub faster_whisper -c int8 -f /path/to/file.mp4
```

For the converter settings:

```bash
podman run --interactive --rm --gpus gpu0 -v %mount%:%mount% -v /home/user/.cache/huggingface/hub:/root/.cache/huggingface/hub -v /path/to/script/transcript_cli.py:/app/transcript_cli.py localhost/faster_whisper -c int8 -l %lang% -f %file%
```

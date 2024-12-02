## Build image:

```bash
podman build -t faster_whisper -f docker/Dockerfile .
```

## Run command:

```bash
docker run -it --name whisper2 --gpus 0 -v /mnt/playout/tv-media:/tv-media -v ~/.cache/huggingface/hub:/root/.cache/huggingface/hub faster_whisper -c int8 -f /path/to/file.mp4
```

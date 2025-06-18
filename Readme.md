### To compile the code for linux, 

configure docker-compose file like this:

```dockerfile
FROM ubuntu:22.04
RUN apt upgrade -y
RUN apt update
RUN apt install ffmpeg libavcodec-dev libavformat-dev libavutil-dev libswresample-dev libswscale-dev libavfilter-dev libavdevice-dev -y
COPY ./target/release/${service_name} ./target/release/${service_name}
ENTRYPOINT ["./target/release/${service_name}"]
```

and add github ci step be like this:
```yaml
    - name: Install FFMPEG
      run: |
        sudo apt-get update
        sudo apt-get install ffmpeg libavcodec-dev libavformat-dev libavutil-dev libswresample-dev libswscale-dev libavfilter-dev libavdevice-dev -y          
```
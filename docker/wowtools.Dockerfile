FROM debian:stable-slim

RUN apt-get update && apt-get install -y curl unzip

RUN mkdir /app
WORKDIR /app
RUN curl -sLO "https://github.com/Marlamin/wow.tools.local/releases/latest/download/Release-linux-x64.zip"
RUN unzip -d . Release-linux-x64.zip
RUN rm Release-linux-x64.zip

HEALTHCHECK CMD curl -f http://127.0.0.1:8080/builds/ || exit 1

ENTRYPOINT ["/app/wow.tools.local"]
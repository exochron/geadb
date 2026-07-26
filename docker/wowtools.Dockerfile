FROM debian:stable-slim

RUN apt-get update && apt-get install -y curl unzip iputils-ping

ENV DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=1
# listen to all IPs not just localhost
ENV ASPNETCORE_URLS="http://0.0.0.0:8080"

RUN mkdir /app
WORKDIR /app
RUN curl -sLO "https://github.com/Marlamin/wow.tools.local/releases/latest/download/Release-linux-x64.zip"
RUN unzip -d . Release-linux-x64.zip
RUN rm Release-linux-x64.zip

HEALTHCHECK CMD curl -f http://127.0.0.1:8080/builds/ || exit 1

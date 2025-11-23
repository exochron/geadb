#FROM mcr.microsoft.com/dotnet/sdk:10.0-aot
#
#RUN git clone --depth 1 --no-tags --single-branch https://github.com/Marlamin/wow.tools.local.git /app
#WORKDIR /app
#RUN git submodule update --init --recursive --depth 1
## patch .net10
#RUN sed -i -e 's/net9.0<\/TargetFrameworks>/net9.0;net10.0<\/TargetFrameworks>/' CascLib/CascLib/CascLib.csproj
#RUN dotnet publish -c Release -o /app --framework net10.0 --property:WarningLevel=0 --self-contained
#
## HEALTHCHECK CMD curl -f http://127.0.0.1:8080/builds/ || exit 1
#
#ENTRYPOINT ["/app/wow.tools.local"]

FROM debian:stable-slim AS Build

RUN apt-get -y update && apt-get install -y curl jq unzip

RUN mkdir /app
WORKDIR /app
RUN curl -s -L -O "$(curl -s -L https://api.github.com/repos/Marlamin/wow.tools.local/releases/latest | jq -r ".assets[].browser_download_url" | grep "linux")"
RUN unzip -d . Release-linux-x64.zip
RUN rm Release-linux-x64.zip

FROM mcr.microsoft.com/dotnet/runtime:latest

COPY --from=Build /app /app
WORKDIR /app
ENTRYPOINT ["/app/wow.tools.local"]
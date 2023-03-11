FROM mcr.microsoft.com/dotnet/sdk:7.0

RUN git clone --depth 1 https://github.com/Marlamin/DBC2CSV.git /app
WORKDIR /app
RUN git submodule update --init --recursive --depth 1
RUN dotnet publish -c Release -o docker --framework net7.0
RUN ln -s /app/WoWDBDefs/definitions /app/docker/definitions

ENTRYPOINT ["/app/docker/DBC2CSV"]

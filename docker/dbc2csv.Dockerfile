FROM mcr.microsoft.com/dotnet/sdk:8.0

RUN git clone --depth 1 https://github.com/Marlamin/DBC2CSV.git /app
WORKDIR /app
RUN git submodule update --init --recursive --depth 1
RUN git submodule update --depth 1 --remote --force WoWDBDefs
RUN dotnet publish -c Release -o /app --framework net8.0

ENTRYPOINT ["/app/DBC2CSV"]

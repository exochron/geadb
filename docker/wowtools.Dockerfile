FROM mcr.microsoft.com/dotnet/sdk:10.0

RUN git clone --depth 1 --no-tags --single-branch https://github.com/Marlamin/wow.tools.local.git /app
WORKDIR /app
RUN git submodule update --init --recursive --depth 1
RUN dotnet publish -c Release -o /app --framework net10.0 --property:WarningLevel=0

HEALTHCHECK CMD curl -f http://127.0.0.1:8080/builds/ || exit 1

ENTRYPOINT ["/app/wow.tools.local"]
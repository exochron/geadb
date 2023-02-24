FROM python:3

RUN apt update && apt install -y git
RUN git clone --depth 1 https://github.com/simulationcraft/simc.git

WORKDIR /simc/casc_extract

RUN pip install -r requirements.txt

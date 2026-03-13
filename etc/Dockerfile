FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y curl vim gawk bison make wget build-essential python3
WORKDIR /opt
ENV GSETTINGS_BACKEND=memory
ENV NO_AT_BRIDGE=1
RUN curl -L https://raw.githubusercontent.com/crhowell3/centurion/v0.1.3/scenario.toml -o scenario.toml
RUN curl -L https://github.com/crhowell3/centurion/releases/download/v0.1.3/centurion_0.1.3_amd64.deb -o centurion.deb \
    && apt install -y ./centurion.deb

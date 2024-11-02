FROM rust:1.81-bookworm AS builder

ARG GITHUB_TOKEN

ARG TAG

ENV GITHUB_ACCESS_TOKEN=${GITHUB_TOKEN}

ENV GH_TOKEN=${GITHUB_TOKEN}

ENV TAG=${TAG}

RUN apt-get update && apt-get install -y build-essential \
  libusb-dev \
  libsdl1.2-dev \
  libfreetype6-dev \
  libunwind-dev \
  curl \
  wget \
  zip \
  unzip \
  protobuf-compiler

RUN curl -Ssf https://pkgx.sh | sh

RUN pkgx install zig@0.13.0 protoc buf deno

COPY . /app

WORKDIR /app

RUN mkdir -p build

WORKDIR /app/webui/rockbox

RUN deno install && deno install --dev

RUN deno task build

WORKDIR /app/build

RUN ../tools/configure --target=sdlapp --type=N --lcdwidth=320 --lcdheight=240 --prefix=/usr/local

RUN make ziginstall -j$(nproc)

RUN deno install -A -r -g https://cli.fluentci.io -n fluentci

ENV PATH=/root/.local/bin:${PATH}

WORKDIR /app

RUN fluentci run --wasm . release

FROM debian:bookworm

RUN apt-get update && apt-get install -y \
  libusb-dev \
  libsdl1.2-dev \
  libfreetype6-dev \
  libunwind-dev \
  alsa-utils \
  libasound2 \
  pulseaudio

COPY --from=builder /usr/local/lib/rockbox /usr/local/lib/rockbox

COPY --from=builder /usr/local/share/rockbox /usr/local/share/rockbox

COPY --from=builder /usr/local/bin/rockboxd /usr/local/bin/rockboxd

ENV SDL_VIDEODRIVER=dummy

ENV SDL_AUDIODRIVER=dummy

EXPOSE 6061
EXPOSE 6062
EXPOSE 6063

CMD ["rockboxd"]

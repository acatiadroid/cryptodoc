FROM rust:latest

RUN apt-get update && apt-get install -y \
    libgtk-3-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/* 

WORKDIR /usr/src/cryptodoc

COPY . .

RUN cargo build --release

WORKDIR /usr/src/cryptodoc/target/release

CMD ["./cryptodoc"]

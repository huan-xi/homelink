FROM clux/muslrust:stable AS chef
RUN apt update -y && apt install -y clang libavcodec-dev libavformat-dev libavutil-dev pkg-config libavfilter-dev libavdevice-dev
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Notice that we are specifying the --target flag!
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin bin

FROM alpine AS runtime
#RUN addgroup -S myuser && adduser -S myuser -G myuser
#USER myuser
WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/bin /app/home-gateway
COPY --from=planner /app/log4rs.yaml /app/log4rs.yaml
COPY --from=planner /app/docker_config.toml /app/config/config.toml
COPY --from=planner /app/dist /app/dist

CMD ["/app/home-gateway"]
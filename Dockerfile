FROM clux/muslrust:stable AS chef
#RUN apt update -y && apt install -y
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
RUN cargo build --release --target x86_64-unknown-linux-musl --bin homelink

FROM alpine AS runtime
ENV DATA_DIR=/data
ENV LOG_DIR=/data
WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/homelink /app/homelink
#COPY --from=planner /app/log4rs.yaml /app/log4rs.yaml
#COPY --from=planner /app/docker_config.toml /app/config.toml
COPY --from=planner /app/dist /app/dist

CMD ["/app/homelink"]
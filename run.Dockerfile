FROM ubuntu AS runtime
RUN apt update && apt install -y ca-certificates
#RUN addgroup -S myuser && adduser -S myuser -G myuser
#USER myuser
WORKDIR /app
COPY ./dist /app/dist
COPY ./log4rs.yaml /app/log4rs.yaml
COPY ./docker_config.toml /app/config/config.toml
COPY ./target/release/bin /app/amps

CMD ["/app/amps"]
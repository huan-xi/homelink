FROM ubuntu AS runtime
#FROM alpine AS runtime
RUN apt update && apt install -y ca-certificates

#RUN apk --no-cache add ca-certificates \
#  && update-ca-certificates
#RUN addgroup -S myuser && adduser -S myuser -G myuser
#USER myuser
WORKDIR /app
COPY ./dist /app/dist
COPY ./templates /templates
COPY ./docker_config.toml /app/config.toml
COPY ./target/x86_64-unknown-linux-musl/release/homelink /app/homelink



CMD ["/app/homelink"]
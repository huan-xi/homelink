FROM ubuntu AS runtime
#FROM alpine AS runtime
RUN apt update && apt install -y ca-certificates

#RUN apk --no-cache add ca-certificates \
#  && update-ca-certificates
#RUN addgroup -S myuser && adduser -S myuser -G myuser
#USER myuser
WORKDIR /app
COPY ./dist /app/dist
COPY ./log4rs.yaml /app/log4rs.yaml
COPY ./docker_config.toml /app/config.toml
COPY ./target/release/bin /app/homelink

CMD ["/app/homelink"]
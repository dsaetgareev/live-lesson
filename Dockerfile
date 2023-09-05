# docker build -t 172.24.17.214:5000/live-lesson:v1 .
FROM rust:slim-bookworm

# Add wasm target
RUN rustup target add wasm32-unknown-unknown 
RUN cargo install --locked trunk

ENV STUN_SERVER_URLS="stun:stun.l.google.com:19302"
ENV TURN_SERVER_URLS="turn:openrelay.metered.ca:80"
ENV TURN_SERVER_USERNAME="openrelay"
ENV TURN_SERVER_CREDENTIAL="openrelay"
ENV SIGNALING_SERVER_URL="ws://172.24.17.214:9001"

WORKDIR /app
COPY . .

EXPOSE 8080

CMD [ "trunk", "serve" ]
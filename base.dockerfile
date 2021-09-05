FROM rust:1.54.0-slim-buster

RUN apt-get -y update &&  apt-get install -y python3 python3-dev build-essential

CMD ["bash"]

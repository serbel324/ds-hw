FROM python:3.8-slim

WORKDIR /http/server

RUN pip install click

COPY server.py .

ENTRYPOINT ["python", "server.py"]

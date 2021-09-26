FROM python:3.8-slim

RUN pip install pytest click

WORKDIR /tests
COPY . .

CMD pytest .
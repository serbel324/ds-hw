FROM python:3.9-slim

RUN pip install flask
RUN pip install pyjwt
RUN pip install loguru

COPY . .

ENTRYPOINT ['python', 'server.py']

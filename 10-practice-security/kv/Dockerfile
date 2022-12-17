FROM python:3.9-slim

RUN pip install flask
RUN pip install pyjwt

COPY . .

ENTRYPOINT ['python', 'server.py']

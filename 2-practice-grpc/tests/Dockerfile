FROM python:3.8-alpine

WORKDIR /practice-grpc/tests
COPY requirements.txt .

RUN pip install -r requirements.txt

COPY *.py .

ENTRYPOINT pytest -vs .

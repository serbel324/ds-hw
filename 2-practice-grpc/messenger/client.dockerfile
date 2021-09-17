FROM python:3.8-slim

WORKDIR /practice-grpc/messenger/client-py
COPY client-py/requirements.txt .
RUN pip install -r requirements.txt

COPY client-py/*.py .

# TODO: copy grpc & proto output files or build right here

ENTRYPOINT ["python", "client.py"]
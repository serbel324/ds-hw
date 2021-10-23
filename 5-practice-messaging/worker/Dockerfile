FROM python:3.8

RUN mkdir /root/app/
WORKDIR /root/app/

RUN pip install gdown
RUN gdown https://drive.google.com/uc?id=1sEDrzH_YgurDrF-8SlBJRtou4CNPOHf4

COPY requirements.txt .
RUN pip install -r requirements.txt && rm -rf /root/.cache/pip

COPY model.py .
RUN python model.py
COPY . .

ENV PYTHONPATH "${PYTHONPATH}:/root/app/"
EXPOSE 5000

CMD ["bash", "-c", "while ! curl -s rabbitmq:15672 > /dev/null; do echo waiting for rabbitmq; sleep 3; done; python worker.py"]

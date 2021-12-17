import time

import requests
from loguru import logger

SERVER_HOST = 'web'
SERVER_PORT = 5000
URL = 'http://' + SERVER_HOST
if SERVER_PORT != 80:
    URL += ':{}'.format(SERVER_PORT)
TEXTS_ENDPOINT = URL + '/texts'


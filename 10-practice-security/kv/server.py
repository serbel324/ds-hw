from flask import Flask
from flask import request, make_response
import json
import jwt
import argparse
import hashlib
from loguru import logger

app = Flask(__name__)

data = {} # 'key' : ('username', 'value')

public_key = None

def gen_cookie(username):
    return jwt.encode(username, private_key, algorithm='RS256')

def hash_password(username, password):
    return hashlib.md5(username + password).hexdigest()

@app.route('/')
def hello_world():
    return 'Hello, World!'

@app.route('/put', methods=['POST'])
def put_handler():
    key = request.args['key']
    value = json.loads(request.data)['value']
    
    cookie = request.cookies.get('jwt')
    if not cookie or cookie is None:
        resp = make_response('A gde?(', 401)
        return resp
    else:
        try:
            username = jwt.decode(cookie, public_key, algorithms=["RS256"])['username']
            data_pair = data.get(key, None)
            if data_pair is None:
                resp = make_response('Created', 200)
                data[key] = (username, value)
                return resp
            else:
                if data_pair[0] == username:
                    data[key] = (username, value)
                    resp = make_response(f'Update', 200)
                    return resp
                else:
                    resp = make_response(f'You are not the owner(', 403)
                    return resp
        except Exception:
            resp = make_response('Bad cookie(', 400)
            return resp

@app.route('/get', methods=['GET'])
def get_handler():
    key = request.args['key']

    cookie = request.cookies.get('jwt')
    if not cookie or cookie is None:
        resp = make_response('A gde?(', 401)
        return resp
    else:
        try:
            username = jwt.decode(cookie, public_key, algorithms=["RS256"])['username']
        except Exception:
            resp = make_response('Bad cookie(', 400)
            return resp
        data_pair = data.get(key, None)
        if data_pair is None:
            resp = make_response('key not found', 404)
            return resp
        else:
            if data_pair[0] == username:
                resp = make_response('{"value": "' + str(data_pair[1]) + '"}', 200)
                return resp
            else:
                resp = make_response(f'You are not the owner(', 403)
                return resp

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--public')
    parser.add_argument('--port')

    args = parser.parse_args()

    f = open(args.public, 'r')
    public_key = f.read()

    app.run(host='0.0.0.0', port=args.port)
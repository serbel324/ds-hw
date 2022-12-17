from flask import Flask
from flask import request, make_response
import jwt
import argparse
import hashlib
import json
from loguru import logger

app = Flask(__name__)

users = {} # 'login' : hashed_password

private_key = None
public_key = None

def gen_cookie(username):
    return jwt.encode({'username': username}, private_key, algorithm='RS256')

def hash_password(username, password):
    return hashlib.md5((username + password).encode('utf-8')).hexdigest()

@app.route('/')
def hello_world():
    return 'Hello, World!'

@app.route('/signup', methods=['POST'])
def signup():
    data = json.loads(request.data)
    username = data['username']
    password = data['password']
    user = users.get(username, None)

    if user is None:
        users[username] = hash_password(username, password)
        resp = make_response('cool password, bro')
        cookie = gen_cookie(username)
        resp.set_cookie('jwt', cookie, max_age=60*60*24*365)
        return resp
    else:
        resp = make_response('Username is already taken', 403)
        return resp

@app.route('/login', methods=['POST'])
def login():
    data = json.loads(request.data)
    username = data['username']
    password = data['password']
    hashed_password = users.get(username, None)

    if hashed_password is None:
        resp = make_response('Username is not found', 403)
        return resp
    else:
        if hash_password(username, password) == hashed_password:
            resp = make_response('Hello')
            cookie = gen_cookie(username)
            resp.set_cookie('jwt', cookie, max_age=60*60*24*365)
            return resp
        else:
            resp = make_response('Wrong password(', 403)
            return resp

@app.route('/whoami', methods=['GET'])
def whoami():
    cookie = request.cookies.get('jwt')
    if not cookie or cookie is None:
        resp = make_response('A gde?(', 401)
        return resp
    else:
        try:
            username = jwt.decode(cookie, public_key, algorithms=["RS256"])['username']
            if users.get(username, None) is None:
                resp = make_response('Bad cookie(', 400)
                return resp
            else:
                resp = make_response(f'Hello, {username}')
                return resp
        except Exception:
            resp = make_response('Bad cookie(', 400)
            return resp


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--private')
    parser.add_argument('--public')
    parser.add_argument('--port')

    args = parser.parse_args()

    with open(args.private, 'r') as f:
        private_key = f.read()
    with open(args.public, 'r') as f:
        public_key = f.read()

    app.run(host='0.0.0.0', port=args.port)
import logging
import pathlib
from dataclasses import dataclass
from socketserver import StreamRequestHandler
import typing as t
import click
import socket

import os
import subprocess
from http_messages import HTTPRequest, HTTPResponse
from http_messages import HTTP_VERSION, TEXT_PLAIN, HEADER_CONTENT_TYPE
from http_messages import HEADER_CONTENT_LENGTH, GET, OK, NOT_FOUND

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__name__)


@dataclass
class HTTPServer:
    server_address: t.Tuple[str, int]
    socket: socket.socket
    server_domain: str
    working_directory: pathlib.Path


class HTTPHandler(StreamRequestHandler):
    server: HTTPServer

    # Use self.rfile and self.wfile to interact with the client
    # Access domain and working directory with self.server.{attr}
    def handle(self) -> None:
        # first_line = self.rfile.readline()
        # logger.info(f"Handle connection from {self.client_address}, first_line {first_line}")
        rfile_content = self.rfile.read()
        http_request = HTTPRequest.from_bytes(b'' + rfile_content)
    
        http_response = HTTPResponse(version="HTTP/" + HTTP_VERSION, status="", headers={}, content="")
        http_response.headers[HEADER_CONTENT_TYPE] = TEXT_PLAIN
    
        request_path = pathlib.Path(http_request.path)
        path = pathlib.Path(self.server.working_directory, request_path)

        if (http_request.method == GET):
            if (path.is_file()):
                f = open(path, 'r')
                http_response.content = f.read()
                http_response.status = OK
            elif (path.is_dir()):
                http_response.content = subprocess.check_output(["ls", "-lA", str(path)])
                http_response.status = OK
            else:
                http_response.content = b'no file lol'
                http_response.status = NOT_FOUND


        http_response.headers[HEADER_CONTENT_LENGTH] = len(http_response.content)

        response = http_response.to_bytes()
        self.wfile.write(response)
        self.wfile.flush()


@click.command()
@click.option("--host", type=str)
@click.option("--port", type=int)
@click.option("--server-domain", type=str)
@click.option("--working-directory", type=str)
def main(host, port, server_domain, working_directory):
    if (host is None):
        host = os.environ.get("SERVER_HOST", "0.0.0.0")
    if (port is None):
        port = os.environ.get("SERVER_PORT", 8080)
    if (server_domain is None):
        server_domain = os.environ.get("SERVER_DOMAIN")
    if (working_directory is None):
        working_directory = os.environ.get("SERVER_WORKING_DIRECTORY")
        if (working_directory is None):
            exit(1)

    working_directory_path = pathlib.Path(working_directory)

    logger.info(
        f"Starting server on {host}:{port}, domain {server_domain}, working directory {working_directory}"
    )

    # Create a server socket
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

    # Set SO_REUSEADDR option
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)

    # Bind the socket object to the address and port
    s.bind((host, port))
    # Start listening for incoming connections
    s.listen()

    logger.info(f"Listening at {s.getsockname()}")
    server = HTTPServer((host, port), s, server_domain, working_directory_path)

    while True:
        # Accept any new connection (request, client_address)
        try:
            conn, addr = s.accept()
        except OSError:
            break

        try:
            # Handle the request
            HTTPHandler(conn, addr, server)

            # Close the connection
            conn.shutdown(socket.SHUT_WR)
            conn.close()
        except Exception as e:
            logger.error(e)
            conn.close()
 
 
if __name__ == "__main__":
    main(auto_envvar_prefix="SERVER")
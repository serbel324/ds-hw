import logging
import pathlib
import dataclasses
from socketserver import BaseRequestHandler
from socketserver import StreamRequestHandler
from socketserver import ThreadingTCPServer
import typing as t
import click

logging.basicConfig(level=logging.DEBUG)

logger = logging.getLogger(__name__)


@dataclasses.dataclass
class HTTPRequest:
    method: str
    path: str
    version: str
    parameters: t.Dict[str, str]
    headers: t.Dict[str, str]
    body: bytes

    @staticmethod
    def from_bytes(data: bytes) -> "HTTPRequest":
        # TODO: Write your code
        pass

    def to_bytes(self) -> bytes:
        # TODO: Write your code
        pass


@dataclasses.dataclass
class HTTPResponse:
    version: str
    status: str
    headers: t.Dict[str, str]
    body: bytes

    @staticmethod
    def from_bytes(data: bytes) -> "HTTPResponse":
        # TODO: Write your code
        pass

    def to_bytes(self) -> bytes:
        # TODO: Write your code
        pass


class HTTPServer(ThreadingTCPServer):
    def __init__(self, host, port, handler_class, server_domain, working_directory):
        super(HTTPServer, self).__init__((host, port), handler_class)
        self.server_domain = server_domain
        self.working_directory = working_directory


class HTTPHandler(StreamRequestHandler):
    server: HTTPServer

    # Use self.rfile and self.wfile to interact with the client
    # Access domain and working directory with self.server.{attr}
    def handle(self) -> None:
        # TODO: Write your code
        pass


@click.command()
@click.option("--host", default="0.0.0.0", type=str)
@click.option("--port", default=10080, type=int)
@click.option("--domain", default="domain.example", type=str)
@click.option("--working-directory", type=pathlib.Path)
def main(host, port, server_domain, working_directory):
    logger.info(
        f"Starting server on {host}:{port}, domain {server_domain}, working directory {working_directory}"
    )

    server = HTTPServer(host, port, HTTPHandler, server_domain, working_directory)

    # TODO: Set socket options for server instance

    server.serve_forever()


if __name__ == "__main__":
    main(auto_envvar_prefix="SERVER")

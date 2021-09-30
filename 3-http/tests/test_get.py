import logging
import stat

import pytest

from http_builder import RequestBuilder


@pytest.fixture(scope="function")
def response(connection):
    request = (
        RequestBuilder()
            .method("GET")
            .path("/")
            .version()
            .hostname(connection.hostname)
            .render()
    )

    response = connection.request(request)
    if not response:
        pytest.fail("Server sent no data to the client")
    yield response


def test_statuscode(grader_http_get_test, connection, response):
    assert response.status == 200, "Response status differs"


def test_server_header(grader_http_get_test, connection, response):
    assert "server" in response.headers, "No `server` header in headers"
    assert response.headers["server"] == connection.hostname, "`server` header does not match configuration"


def test_another_header(grader_http_get_test, connection):
    request = (
        RequestBuilder()
            .method("GET")
            .path("/")
            .version()
            .hostname("not-a-valid-hostname")
            .render()
    )

    response = connection.request(request)
    assert response, "Server sent no data to the client"
    assert response.status == 400, "Server responded with wrong code, expected 400"


def test_content_headers(grader_http_get_test, connection, response):
    assert "content-type" in response.headers, 'No `content-type` header in response headers'
    assert "content-length" in response.headers, 'No `content-length` header in response headers'
    assert response.body, "No body in response from server"
    assert len(response.body) == int(response.headers["content-length"]), "`content-length` value does not match with actual body length"


def test_response_body_data_format(
    grader_http_get_test, connection, response, file_state
):
    body = response.body.decode()

    lines = body.split("\n")
    assert len(lines) == file_state.count("/")

    for line in lines:
        assert len(line.split()) == 7


def test_response_body_permissions(
    grader_http_get_test, connection, response, file_state
):
    for line in response.body.decode().split("\r\n"):
        permissions, *_, name = line.split()
        file_on_disk = file_state[f"/{name}"]

        assert permissions[0] == ("d" if file_on_disk.is_dir() else "-")
        assert permissions[1] == "r" if file_on_disk.stat()[0] & stat.S_IRUSR else "-"
        assert permissions[2] == "w" if file_on_disk.stat()[0] & stat.S_IWUSR else "-"
        assert permissions[3] == "x" if file_on_disk.stat()[0] & stat.S_IXUSR else "-"
        assert permissions[4] == "r" if file_on_disk.stat()[0] & stat.S_IRGRP else "-"
        assert permissions[5] == "w" if file_on_disk.stat()[0] & stat.S_IWGRP else "-"
        assert permissions[6] == "x" if file_on_disk.stat()[0] & stat.S_IXGRP else "-"
        assert permissions[7] == "r" if file_on_disk.stat()[0] & stat.S_IROTH else "-"
        assert permissions[8] == "w" if file_on_disk.stat()[0] & stat.S_IWOTH else "-"
        assert permissions[9] == "x" if file_on_disk.stat()[0] & stat.S_IXOTH else "-"

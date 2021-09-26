import stat

import pytest

from http import RequestBuilder


@pytest.fixture(scope="module")
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
    yield response


def test_statuscode(grader_http_get_test, connection, response):
    assert response.status == 200


def test_server_header(grader_http_get_test, connection, response):
    assert "server" in response.headers
    assert response.headers["server"] == connection.hostname


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

    assert response.status == 400


def test_content_headers(grader_http_get_test, connection, response):
    assert "content-type" in response.headers
    assert "content-length" in response.headers
    assert response.body
    assert len(response.body) == int(response.headers['content-length'])


def test_response_body_data_format(grader_http_get_test, connection, response, file_state):
    body = response.body.decode()

    lines = body.split('\n')
    assert len(lines) == file_state.count('/')

    for line in lines:
        assert len(line.split()) == 7


def test_response_body_permissions(grader_http_get_test, connection, response, file_state):
    for line in response.body.decode().split():
        permissions, *_, name = line.split()
        file_on_disk = file_state[f'/{name}']

        assert permissions[0] == ('d' if file_on_disk.is_dir() else '-')
        assert permissions[1] == 'r' if file_on_disk.stat()[0] & stat.S_IRUSR else '-'
        assert permissions[2] == 'w' if file_on_disk.stat()[0] & stat.S_IWUSR else '-'
        assert permissions[3] == 'x' if file_on_disk.stat()[0] & stat.S_IXUSR else '-'
        assert permissions[4] == 'r' if file_on_disk.stat()[0] & stat.S_IRGRP else '-'
        assert permissions[5] == 'w' if file_on_disk.stat()[0] & stat.S_IWGRP else '-'
        assert permissions[6] == 'x' if file_on_disk.stat()[0] & stat.S_IXGRP else '-'
        assert permissions[7] == 'r' if file_on_disk.stat()[0] & stat.S_IROTH else '-'
        assert permissions[8] == 'w' if file_on_disk.stat()[0] & stat.S_IWOTH else '-'
        assert permissions[9] == 'x' if file_on_disk.stat()[0] & stat.S_IXOTH else '-'

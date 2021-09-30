from http_builder import RequestBuilder
from tests.test_put import LOREM_IPSUM
from shutil import rmtree


def test_delete_file(grader_http_delete_test, connection, file_state):
    file_state['/test-file'].unlink(missing_ok=True)
    file_state['/test-file'].write_text(LOREM_IPSUM)

    request = (
        RequestBuilder()
            .method("DELETE")
            .path("/test-file")
            .version()
            .hostname(connection.hostname)
            .render()
    )

    response = connection.request(request)

    assert response.status == 200
    assert not file_state['/test-file'].exists()


def test_delete_directory_fail(grader_http_delete_test, connection, file_state):
    rmtree(file_state['/test-directory'])
    file_state['/test-directory'].mkdir(parents=True, exist_ok=True)

    request = (
        RequestBuilder()
            .method("DELETE")
            .path("/test-directory")
            .version()
            .hostname(connection.hostname)
            .render()
    )

    response = connection.request(request)

    assert response.status == 406
    assert file_state['/test-directory'].exists()
    assert file_state['/test-directory'].is_dir()


def test_delete_directory_success(grader_http_delete_test, connection, file_state):
    rmtree(file_state['/test-directory'])
    file_state['/test-directory'].mkdir(parents=True, exist_ok=True)

    request = (
        RequestBuilder()
            .method("DELETE")
            .path("/test-directory")
            .version()
            .hostname(connection.hostname)
            .header('Remove-Directory', 'True')
            .render()
    )

    response = connection.request(request)

    assert response.status == 200
    assert not file_state['/test-directory'].exists()


def test_update_data_in_file(grader_http_put_test, connection, file_state):
    file_state["/test-file"].unlink(missing_ok=True)
    file_state["/test-file"].write_text("Hello, world!")

    request = (
        RequestBuilder()
            .method("PUT")
            .path("/test-file")
            .version()
            .hostname(connection.hostname)
            .body(LOREM_IPSUM)
            .render()
    )

    response = connection.request(request)

    assert response.status == 200

    assert file_state['/test-file'].read_text() == LOREM_IPSUM

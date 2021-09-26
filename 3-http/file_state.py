import pathlib


class FileState:
    def __init__(self, base_path):
        self._base_path = base_path
        self._files = dict()

    def count(self, path: pathlib.Path):
        return len(list(self._to_actual_path(path).glob("*")))

    def _to_virtual_path(self, path) -> pathlib.Path:
        return pathlib.Path("/") / pathlib.Path(path).relative_to(self._base_path)

    def _to_actual_path(self, path) -> pathlib.Path:
        return self._base_path / pathlib.Path(path).relative_to("/")

    def __getitem__(self, item):
        return self._to_actual_path(item)
